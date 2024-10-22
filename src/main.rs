#![no_main]
#![no_std]
#![feature(ptr_as_ref_unchecked)]

// Required for panic handler
extern crate flipperzero_rt;

// Required for allocating memory on the heap
extern crate alloc;
extern crate flipperzero_alloc;

use core::{cmp, ffi::CStr};

use alloc::{borrow::ToOwned, string::String, sync::Arc};
use flipperzero::{
    furi::{self, string::FuriString, thread},
    print, println,
};
use flipperzero_rt::{entry, manifest};
use serial_marker::Initialized;
use ufmt::derive::uDebug;

use safe::*;

mod cmd;
mod safe;

manifest!(name = "RG-15");
entry!(main);

const MAIN_VIEW_ID: u32 = 0;
const RAW_VIEW_ID: u32 = 1;
const CMD_VIEW_ID: u32 = 2;

const SCREEN_HEIGHT: u32 = 64;
const SCREEN_WIDTH: u32 = 128;

const CMD_SUBMENU_HEADER: &'static CStr = c"Command to RG-15";

fn main(_args: Option<&CStr>) -> i32 {
    let gui = Gui::open();
    let mut view_dispatcher = ViewDispatcher::new(gui, ViewDispatcherType::Fullscreen);
    let view_switcher = view_dispatcher.view_switcher();

    let Some(serial_handle) = SerialHandle::acquire(SerialId::Lpuart) else {
        return 1;
    };
    let mut serial_handle: SerialHandle<_> = serial_handle.init(9600);
    let rx = serial_handle.async_rx_start(false);

    let context = Arc::new(CallbackContext {
        serial_handle,
        view_switcher,
    });

    let mut main_view = View::new();
    main_view.set_context(context.clone());
    main_view.create_model::<Data>();
    main_view.set_previous_callback::<ViewNone>();
    main_view.set_draw_callback::<MainView>();
    main_view.set_input_callback::<MainView>();
    let main_view = view_dispatcher.add_view(main_view, MAIN_VIEW_ID);
    view_dispatcher.switch_to_view(0);

    let mut raw_widget = Widget::new();
    let raw_view = raw_widget.as_mut_view();
    raw_view.set_previous_callback::<OtherView>();
    let raw_widget = view_dispatcher.add_widget_mutex(raw_widget, RAW_VIEW_ID);

    let mut cmd_submenu = Submenu::new();
    let cmd_view = cmd_submenu.as_mut_view();
    cmd_view.set_previous_callback::<OtherView>();
    cmd_submenu.set_header(CMD_SUBMENU_HEADER);
    for cmd in cmd::Command::list() {
        cmd_submenu.add_item::<CmdSubmenuItem, _>(cmd.name(), cmd.code(), Some(context.clone()));
    }
    view_dispatcher.add_submenu(cmd_submenu, CMD_VIEW_ID);

    let rx_thread = furi::thread::Builder::new()
        .stack_size(8192)
        .spawn(move || {
            let mut buf = [0u8; 4096 + 1];
            let mut i = 0;

            while rx.is_sender_alive() {
                // keep it in here to properly destroy the view if not needed anymore
                let Some(view) = main_view.upgrade() else { return 0 };

                let mut byte = [0u8];
                let received =
                    rx.recv_with_timeout(&mut byte, furi::time::Duration::from_millis(200));
                if received == 0 {
                    continue;
                }

                let byte = byte[0];
                buf[i] = byte;
                i = cmp::min(i + 1, 4096);

                if byte != b'\n' {
                    continue;
                }
                let line = String::from_utf8(buf[0..i].to_owned());

                // reset buffer
                buf.fill(0);
                i = 0;

                let Ok(line) = line else { continue };
                if let Some(mut model) = view.get_model::<Data>() {
                    let model = &mut model.model;

                    let line = line.trim();
                    model.raw.push_str(line);
                    model.raw.push('\n');
                    let split = line.split(',');
                    for split in split {
                        let split = split.trim();
                        let mut split = split.splitn(2, ' ');
                        let Some(key) = split.next() else { continue };
                        let Some(value) = split.next() else { continue };
                        match key {
                            "Acc" => model.acc = FuriString::from(value),
                            "EventAcc" => model.event_acc = FuriString::from(value),
                            "TotalAcc" => model.total_acc = FuriString::from(value),
                            "RInt" => model.r_int = FuriString::from(value),
                            _ => continue,
                        }
                    }

                    let Some(raw_widget) = raw_widget.upgrade() else { continue };
                    let mut raw_widget = raw_widget.lock();
                    raw_widget.reset();
                    raw_widget.add_text_scroll_element(0, 0, SCREEN_WIDTH as u8, SCREEN_HEIGHT as u8, model.raw.as_c_str());
                };
            }

            0
        });

    println!("we run now!");
    view_dispatcher.run();
    drop(view_dispatcher);
    rx_thread.join();

    0
}

#[derive(Debug, uDebug)]
struct Data {
    raw: FuriString,
    acc: FuriString,
    event_acc: FuriString,
    total_acc: FuriString,
    r_int: FuriString,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            raw: FuriString::from(""),
            acc: FuriString::from("acc"),
            event_acc: FuriString::from("event_acc"),
            total_acc: FuriString::from("total_acc"),
            r_int: FuriString::from("r_int"),
        }
    }
}

struct CallbackContext {
    serial_handle: SerialHandle<Initialized>,
    view_switcher: ViewSwitcher
}

struct MainView;

impl ViewDrawCallback for MainView {
    type Model = Data;

    fn callback(canvas: &mut Canvas, model: Option<&Self::Model>) {
        canvas.elements_button_right(c"raw");
        canvas.elements_button_left(c"cmd");

        const POLL_WIDTH: u32 = 34;
        const POLL_HEIGHT: u32 = 14;
        let box_x = ((SCREEN_WIDTH - POLL_WIDTH) / 2) as i32;
        let box_y = ((SCREEN_HEIGHT - POLL_HEIGHT) + 2) as i32;

        let down_arrow = |canvas: &mut Canvas, x, y| {
            for yi in 0..4 {
                for xi in (0 + yi)..(7 - yi) {
                    canvas.draw_dot(x + xi, y + yi);
                }
            }
        };

        canvas.draw_rbox(
            box_x as i32,
            box_y as i32,
            POLL_WIDTH as usize,
            POLL_HEIGHT as usize,
            3,
        );
        canvas.invert_color();
        down_arrow(canvas, box_x + 5, box_y + 4);
        canvas.draw_str(box_x + 15, box_y + 9, c"poll");
        canvas.invert_color();

        if let Some(data) = model {
            [
                c"last acc:",
                c"event acc:",
                c"total acc:",
                c"rain int:",
            ]
            .iter()
            .enumerate()
            .map(|(i, s)| (s, (i as i32 + 1) * 11))
            .zip([&data.acc, &data.event_acc, &data.total_acc, &data.r_int].into_iter())
            .for_each(|((label, y), data)| {
                let padding = 10;
                canvas.draw_str(padding, y, label);
                let data = data.as_c_str();
                let data_width = canvas.string_width(data) as u32;
                canvas.draw_str((SCREEN_WIDTH - data_width) as i32 - padding, y, data);
            });
        }
    }
}

impl ViewInputCallback for MainView {
    type Context = CallbackContext;

    fn callback(input_key: InputKey, context: Option<&Self::Context>) -> bool {
        let Some(context) = context else { return false };
        match input_key {
            InputKey::Down => context.serial_handle.tx(c"r\r\n".to_bytes()),
            InputKey::Right => context.view_switcher.switch_to_view(RAW_VIEW_ID),
            InputKey::Left => context.view_switcher.switch_to_view(CMD_VIEW_ID),
            _ => return false
        }

        true
    }
}

struct OtherView;

impl ViewNavigationCallback for OtherView {
    type Context = ();

    fn callback(_: Option<&Self::Context>) -> u32 {
        MAIN_VIEW_ID
    }
}

struct CmdSubmenuItem;

impl SubmenuItem for CmdSubmenuItem {
    type Context = CallbackContext;

    fn select(context: &Self::Context, code: u32) {
        let Some(cmd) = cmd::Command::try_from_code(code) else { return };
        context.serial_handle.tx(cmd.cmd().as_bytes());
        context.view_switcher.switch_to_view(MAIN_VIEW_ID);
    }
}

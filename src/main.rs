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

fn main(_args: Option<&CStr>) -> i32 {
    let gui = Gui::open();
    let mut view_dispatcher = ViewDispatcher::new(gui, ViewDispatcherType::Fullscreen);

    let Some(serial_handle) = SerialHandle::acquire(SerialId::Lpuart) else {
        return 1;
    };
    let mut serial_handle: SerialHandle<_> = serial_handle.init(9600);
    let rx = serial_handle.async_rx_start(false);
    let serial_handle = Arc::new(serial_handle);

    let mut view = View::new();
    view.set_context(serial_handle);
    view.create_model::<Data>();
    view.set_previous_callback::<ViewNone>();
    view.set_draw_callback::<MainView>();
    view.set_input_callback::<MainView>();
    let view = view_dispatcher.add_view(view, 0);
    view_dispatcher.switch_to_view(0);

    let rx_thread = furi::thread::Builder::new()
        .stack_size(8192)
        .spawn(move || {
            let mut buf = [0u8; 4096 + 1];
            let mut i = 0;

            while rx.is_sender_alive() {
                // keep it in here to properly destroy the view if not needed anymore
                let Some(view) = view.upgrade() else { return 0 };

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
            raw: FuriString::from("ja, lol ey"),
            acc: FuriString::from("acc"),
            event_acc: FuriString::from("event_acc"),
            total_acc: FuriString::from("total_acc"),
            r_int: FuriString::from("r_int"),
        }
    }
}

struct MainView;

impl ViewDrawCallback for MainView {
    type Model = Data;

    fn callback(canvas: &mut Canvas, model: Option<&Self::Model>) {
        const SCREEN_HEIGHT: u32 = 64;
        const SCREEN_WIDTH: u32 = 128;

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
                c"last accumulated:",
                c"event accumulated:",
                c"total accumulated:",
                c"rain intensity:",
            ]
            .iter()
            .enumerate()
            .map(|(i, s)| (s, (i as i32 + 1) * 11))
            .zip([&data.acc, &data.event_acc, &data.total_acc, &data.r_int].into_iter())
            .for_each(|((label, y), data)| {
                canvas.draw_str(0, y, label);
                let data = data.as_c_str();
                let data_width = canvas.string_width(data) as u32;
                canvas.draw_str((SCREEN_WIDTH - data_width) as i32, y, data);
            });
        }
    }
}

impl ViewInputCallback for MainView {
    type Context = SerialHandle<Initialized>;

    fn callback(input_key: InputKey, context: Option<&Self::Context>) -> bool {
        let Some(context) = context else { return false };
        if input_key == InputKey::Down {
            context.tx(c"r\r\n".to_bytes());
            return true;
        }

        false
    }
}

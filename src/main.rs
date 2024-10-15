#![no_main]
#![no_std]
#![feature(ptr_as_ref_unchecked)]
#![allow(unused)]

// Required for panic handler
extern crate flipperzero_rt;

// Required for allocating memory on the heap
extern crate alloc;
extern crate flipperzero_alloc;

use core::{
    ffi::{c_void, CStr},
    mem::{self, MaybeUninit},
    ops::Deref,
    ptr::{self, NonNull},
};

use alloc::sync::Arc;

use flipperzero::{
    furi::string::FuriString,
    gui::canvas::Align,
    io::Write,
    println,
    storage::{File, OpenOptions},
};
use flipperzero_rt::{entry, manifest};

use flipperzero_sys::{self as sys, furi::UnsafeRecord, FuriHalSerialRxEvent};

mod cmd;
mod safe;

manifest!(name = "RG-15");
entry!(main);

struct App {
    _gui: UnsafeRecord<sys::Gui>,
    view_dispatcher: NonNull<sys::ViewDispatcher>,
    views: AppViews,

    serial_handle: NonNull<sys::FuriHalSerialHandle>,
    rx_stream: NonNull<sys::FuriStreamBuffer>,
}

struct AppData {
    raw: FuriString,
    acc: FuriString,
    event_acc: FuriString,
    total_acc: FuriString,
    r_int: FuriString,
}

struct AppViews {
    baud_rate: NonNull<sys::Submenu>,
    main: NonNull<sys::View>,
    cmd: NonNull<sys::Submenu>,
    raw: NonNull<sys::Widget>,
}

impl AppViews {
    const BAUD_RATE_VIEW_ID: u32 = 0;
    const MAIN_VIEW_ID: u32 = 1;
    const CMD_VIEW_ID: u32 = 2;
    const RAW_VIEW_ID: u32 = 3;

    const NONE_VIEW_ID: u32 = 0xFFFFFFFF;
    #[no_mangle]
    unsafe extern "C" fn exit(_: *mut c_void) -> u32 {
        Self::NONE_VIEW_ID
    }

    #[no_mangle]
    unsafe extern "C" fn to_main(_: *mut c_void) -> u32 {
        Self::MAIN_VIEW_ID
    }

    const BAUD_RATE_SUBMENU_HEADER: &'static CStr = c"Select Baud Rate";
    #[no_mangle]
    unsafe extern "C" fn baud_rate_submenu_select_callback(context: *mut c_void, baud_rate: u32) {
        let app: *mut App = context.cast();
        println!("app in callback: {:?}", app);
        let app = app.as_mut_unchecked();

        let baud_rate = cmd::BaudRate::try_from(baud_rate as u16).unwrap();
        sys::furi_hal_serial_deinit(app.serial_handle.as_ptr());
        sys::furi_hal_serial_init(app.serial_handle.as_ptr(), baud_rate.rate() as u32);
        sys::furi_hal_serial_async_rx_start(
            app.serial_handle.as_ptr(),
            Some(serial_rx_callback),
            context,
            false,
        );

        sys::view_dispatcher_switch_to_view(app.view_dispatcher.as_ptr(), AppViews::MAIN_VIEW_ID);
    }

    #[no_mangle]
    unsafe extern "C" fn main_view_draw_callback(canvas: *mut sys::Canvas, data: *mut c_void) {
        // TODO: use data
        let data: *mut AppData = data.cast();
        let data: &mut AppData = data.as_mut_unchecked();

        const SCREEN_HEIGHT: u32 = 64;
        const SCREEN_WIDTH: u32 = 128;

        sys::elements_button_right(canvas, c"raw".as_ptr());
        sys::elements_button_left(canvas, c"cmd".as_ptr());

        const POLL_WIDTH: u32 = 34;
        const POLL_HEIGHT: u32 = 14;
        let box_x = ((SCREEN_WIDTH - POLL_WIDTH) / 2) as i32;
        let box_y = ((SCREEN_HEIGHT - POLL_HEIGHT) + 2) as i32;

        let down_arrow = |canvas, x, y| {
            for yi in 0..4 {
                for xi in (0 + yi)..(7 - yi) {
                    sys::canvas_draw_dot(canvas, x + xi, y + yi);
                }
            }
        };

        sys::canvas_draw_rbox(
            canvas,
            box_x as i32,
            box_y as i32,
            POLL_WIDTH as usize,
            POLL_HEIGHT as usize,
            3,
        );
        sys::canvas_invert_color(canvas);
        down_arrow(canvas, box_x + 5, box_y + 4);
        sys::canvas_draw_str(canvas, box_x + 15, box_y + 9, c"poll".as_ptr());
        sys::canvas_invert_color(canvas);

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
            sys::canvas_draw_str(canvas, 0, y, label.as_ptr());
            let data = data.as_c_ptr();
            let data_width = sys::canvas_string_width(canvas, data) as u32;
            sys::canvas_draw_str(canvas, (SCREEN_WIDTH - data_width) as i32, y, data);
        });
    }

    #[no_mangle]
    unsafe extern "C" fn main_view_input_callback(
        event: *mut sys::InputEvent,
        app: *mut c_void,
    ) -> bool {
        debug_assert!(!event.is_null());
        debug_assert!(!app.is_null());

        let app: *const App = app.cast_const().cast();
        let app = app.as_ref_unchecked();

        let event: &mut sys::InputEvent = event.as_mut_unchecked();
        match event.key {
            sys::InputKey_InputKeyLeft => sys::view_dispatcher_switch_to_view(
                app.view_dispatcher.as_ptr(),
                AppViews::CMD_VIEW_ID,
            ),
            sys::InputKey_InputKeyRight => sys::view_dispatcher_switch_to_view(
                app.view_dispatcher.as_ptr(),
                AppViews::RAW_VIEW_ID,
            ),
            sys::InputKey_InputKeyDown => sys::furi_hal_serial_tx(
                app.serial_handle.as_ptr(),
                c"r\n".as_ptr().cast(),
                c"r\n".count_bytes(),
            ),
            _ => return false,
        }

        true
    }

    const CMD_SUBMENU_HEADER: &'static CStr = c"Command to RG-15";
}

#[no_mangle]
pub unsafe extern "C" fn serial_rx_callback(
    handle: *mut sys::FuriHalSerialHandle,
    event: sys::FuriHalSerialRxEvent,
    context: *mut c_void,
) {
    println!("event!");
    return;

    if event != sys::FuriHalSerialRxEvent_FuriHalSerialRxEventData {
        // TODO: maybe handle other variants
        println!("some other event");
        return;
    }

    let app: *mut App = context.cast();
    let app: &mut App = app.as_mut_unchecked();

    let byte = sys::furi_hal_serial_async_rx(handle);
    let data: *const c_void = (&byte as *const u8).cast();
    sys::furi_stream_buffer_send(app.rx_stream.as_ptr(), data, 1, 0);

    return;

    println!("we have a line");
    if byte == b'\n' {
        let mut buf = [0u8; 128];
        let buf_len = sys::furi_stream_buffer_receive(
            app.rx_stream.as_ptr(),
            (&mut buf as *mut u8).cast(),
            128,
            0,
        );
        let line = CStr::from_bytes_until_nul(&buf).unwrap();
        let Ok(line) = line.to_str() else { return };

        let data = sys::view_get_model(app.views.main.as_ptr());
        let data: *mut AppData = data.cast();
        let data: &mut AppData = data.as_mut_unchecked();

        data.raw.push_str(line);
        for segment in line.lines().map(|line| line.split(",")).flatten() {
            let segment = segment.trim();
            let mut split = segment.splitn(1, ' ');
            let Some(key) = split.next() else { continue };
            let Some(value) = split.next() else { continue };

            match key {
                "Acc" => data.acc = FuriString::from(value),
                "EventAcc" => data.event_acc = FuriString::from(value),
                "TotalAcc" => data.total_acc = FuriString::from(value),
                "RInt" => data.r_int = FuriString::from(value),
                _ => continue,
            }
        }

        sys::view_commit_model(app.views.main.as_ptr(), true);
    }
}

impl App {
    fn new() -> Arc<Self> {
        println!("allocating app");
        unsafe {
            let mut app: Arc<MaybeUninit<App>> = Arc::new_uninit();
            println!("app in alloc: {:?}", app.as_ptr());

            let gui = UnsafeRecord::open(c"gui".as_ptr());
            let view_dispatcher = NonNull::new_unchecked(sys::view_dispatcher_alloc());
            sys::view_dispatcher_attach_to_gui(
                view_dispatcher.as_ptr(),
                gui.as_ptr(),
                sys::ViewDispatcherType_ViewDispatcherTypeFullscreen,
            );

            let baud_rate_submenu = NonNull::new_unchecked(sys::submenu_alloc());
            sys::submenu_set_header(
                baud_rate_submenu.as_ptr(),
                AppViews::BAUD_RATE_SUBMENU_HEADER.as_ptr(),
            );
            for baud_rate in cmd::BaudRate::list() {
                sys::submenu_add_item(
                    baud_rate_submenu.as_ptr(),
                    baud_rate.rate_as_char().as_ptr(),
                    baud_rate.rate() as u32,
                    Some(AppViews::baud_rate_submenu_select_callback),
                    app.as_ptr().cast_mut().cast(),
                );
            }
            sys::submenu_set_selected_item(
                baud_rate_submenu.as_ptr(),
                cmd::BaudRate::default().rate() as u32,
            );
            let baud_rate_view = sys::submenu_get_view(baud_rate_submenu.as_ptr());
            sys::view_dispatcher_add_view(
                view_dispatcher.as_ptr(),
                AppViews::BAUD_RATE_VIEW_ID,
                baud_rate_view,
            );
            sys::view_set_previous_callback(baud_rate_view, Some(AppViews::exit));
            sys::view_dispatcher_switch_to_view(
                view_dispatcher.as_ptr(),
                AppViews::BAUD_RATE_VIEW_ID,
            );

            let main = NonNull::new_unchecked(sys::view_alloc());
            sys::view_set_previous_callback(main.as_ptr(), Some(AppViews::exit));
            sys::view_allocate_model(
                main.as_ptr(),
                sys::ViewModelType_ViewModelTypeLocking,
                mem::size_of::<AppData>(),
            );
            // memory is allocated but not initialized, hence MaybeUninit
            let app_data: *mut MaybeUninit<AppData> = sys::view_get_model(main.as_ptr()).cast();
            let mut app_data = app_data.as_mut_unchecked();
            app_data.write(AppData {
                raw: FuriString::from("ja, lol ey"),
                acc: FuriString::from("acc"),
                event_acc: FuriString::from("event_acc"),
                total_acc: FuriString::from("total_acc"),
                r_int: FuriString::from("r_int"),
            });
            sys::view_commit_model(main.as_ptr(), true);
            sys::view_set_draw_callback(main.as_ptr(), Some(AppViews::main_view_draw_callback));
            sys::view_set_context(main.as_ptr(), app.as_ptr().cast_mut().cast());
            sys::view_set_input_callback(main.as_ptr(), Some(AppViews::main_view_input_callback));
            sys::view_dispatcher_add_view(
                view_dispatcher.as_ptr(),
                AppViews::MAIN_VIEW_ID,
                main.as_ptr(),
            );

            let cmd_submenu = NonNull::new_unchecked(sys::submenu_alloc());
            sys::submenu_set_header(cmd_submenu.as_ptr(), AppViews::CMD_SUBMENU_HEADER.as_ptr());
            for (i, command) in cmd::Command::list().into_iter().enumerate() {
                sys::submenu_add_item(
                    cmd_submenu.as_ptr(),
                    command.name().as_ptr(),
                    i as u32,
                    None,
                    ptr::null_mut(),
                );
            }
            let cmd_view = sys::submenu_get_view(cmd_submenu.as_ptr());
            sys::view_dispatcher_add_view(
                view_dispatcher.as_ptr(),
                AppViews::CMD_VIEW_ID,
                cmd_view,
            );
            sys::view_set_previous_callback(cmd_view, Some(AppViews::to_main));

            let raw_widget = NonNull::new_unchecked(sys::widget_alloc());
            let raw_view = sys::widget_get_view(raw_widget.as_ptr());
            let app_data: *mut AppData = sys::view_get_model(main.as_ptr()).cast();
            let app_data: &mut AppData = app_data.as_mut_unchecked();
            sys::widget_add_text_scroll_element(
                raw_widget.as_ptr(),
                5,
                5,
                128 - 5,
                64 - 5,
                app_data.raw.as_c_ptr(),
            );
            sys::view_commit_model(main.as_ptr(), false);
            sys::view_dispatcher_add_view(
                view_dispatcher.as_ptr(),
                AppViews::RAW_VIEW_ID,
                raw_view,
            );
            sys::view_set_previous_callback(raw_view, Some(AppViews::to_main));

            let views = AppViews {
                baud_rate: baud_rate_submenu,
                main,
                cmd: cmd_submenu,
                raw: raw_widget,
            };

            let serial_handle = NonNull::new(sys::furi_hal_serial_control_acquire(
                sys::FuriHalSerialId_FuriHalSerialIdUsart,
            ))
            .unwrap();
            let rx_stream = NonNull::new_unchecked(sys::furi_stream_buffer_alloc(128, 1));

            Arc::get_mut(&mut app).unwrap().write(App {
                _gui: gui, // hold it for some reason
                view_dispatcher,
                views,

                serial_handle,
                rx_stream,
            });
            app.assume_init()
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        println!("dropping app");
        unsafe {
            sys::furi_hal_serial_deinit(self.serial_handle.as_ptr());
            sys::furi_hal_serial_control_release(self.serial_handle.as_ptr());

            sys::furi_stream_buffer_free(self.rx_stream.as_ptr());

            // free views
            sys::view_dispatcher_remove_view(
                self.view_dispatcher.as_ptr(),
                AppViews::BAUD_RATE_VIEW_ID,
            );
            sys::submenu_free(self.views.baud_rate.as_ptr());

            sys::view_dispatcher_remove_view(self.view_dispatcher.as_ptr(), AppViews::MAIN_VIEW_ID);
            sys::view_free_model(self.views.main.as_ptr());
            sys::view_free(self.views.main.as_ptr());

            sys::view_dispatcher_remove_view(self.view_dispatcher.as_ptr(), AppViews::CMD_VIEW_ID);
            sys::submenu_free(self.views.cmd.as_ptr());

            sys::view_dispatcher_remove_view(self.view_dispatcher.as_ptr(), AppViews::RAW_VIEW_ID);
            sys::widget_free(self.views.raw.as_ptr());

            sys::view_dispatcher_free(self.view_dispatcher.as_ptr());
        }
    }
}

fn main(_args: Option<&CStr>) -> i32 {
    let app = App::new();
    println!("app after alloc: {:?}", app.deref() as *const App);

    unsafe {
        sys::view_dispatcher_run(app.view_dispatcher.as_ptr());
    }

    0
}

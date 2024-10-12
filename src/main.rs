#![no_main]
#![no_std]
#![feature(ptr_as_ref_unchecked)]

// Required for panic handler
extern crate flipperzero_rt;

// Required for allocating memory on the heap
extern crate alloc;
extern crate flipperzero_alloc;

use core::{
    ffi::{c_void, CStr},
    mem::MaybeUninit,
    ops::Deref,
    ptr::{self, NonNull},
};

use alloc::sync::Arc;

use flipperzero::{
    io::Write,
    println,
    storage::{File, OpenOptions},
};
use flipperzero_rt::{entry, manifest};

use flipperzero_sys::{self as sys, furi::UnsafeRecord};

mod cmd;

manifest!(name = "RG-15");
entry!(main);

struct App {
    gui: UnsafeRecord<sys::Gui>,
    view_dispatcher: NonNull<sys::ViewDispatcher>,
    views: AppViews,
}

struct AppViews {
    baud_rate: NonNull<sys::Submenu>,
    main: NonNull<sys::View>,
}

impl AppViews {
    const BAUD_RATE_VIEW_ID: u32 = 0;
    const MAIN_VIEW_ID: u32 = 1;

    const NONE_VIEW_ID: u32 = 0xFFFFFFFF;
    #[no_mangle]
    unsafe extern "C" fn exit(_: *mut c_void) -> u32 {
        Self::NONE_VIEW_ID
    }

    const BAUD_RATE_SUBMENU_HEADER: &'static CStr = c"Select Baud Rate";
    #[no_mangle]
    unsafe extern "C" fn baud_rate_submenu_select_callback(app: *mut c_void, baud_rate: u32) {
        let app: *const App = app.cast_const().cast();
        println!("app in callback: {:?}", app);
        let app = app.as_ref_unchecked();
        let _ = app;
        println!("selected: {}", baud_rate);
        // TODO: actually select the baud rate
        sys::view_dispatcher_switch_to_view(app.view_dispatcher.as_ptr(), AppViews::MAIN_VIEW_ID);
    }

    #[no_mangle]
    unsafe extern "C" fn main_view_draw_callback(canvas: *mut sys::Canvas, model: *mut c_void) {
        println!("look at me, I draw");
        sys::elements_button_down(canvas, c"poll".as_ptr());
        sys::elements_button_up(canvas, c"cmd".as_ptr());
        sys::elements_button_left(canvas, c"raw".as_ptr());
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
            sys::view_set_draw_callback(main.as_ptr(), Some(AppViews::main_view_draw_callback));
            sys::view_dispatcher_add_view(
                view_dispatcher.as_ptr(),
                AppViews::MAIN_VIEW_ID,
                main.as_ptr(),
            );

            let views = AppViews {
                baud_rate: baud_rate_submenu,
                main,
            };

            Arc::get_mut(&mut app).unwrap().write(App {
                gui,
                view_dispatcher,
                views,
            });
            app.assume_init()
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        println!("dropping app");
        unsafe {
            // free views
            sys::view_dispatcher_remove_view(
                self.view_dispatcher.as_ptr(),
                AppViews::BAUD_RATE_VIEW_ID,
            );
            sys::submenu_free(self.views.baud_rate.as_ptr());

            sys::view_dispatcher_remove_view(self.view_dispatcher.as_ptr(), AppViews::MAIN_VIEW_ID);
            sys::view_free(self.views.main.as_ptr());

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

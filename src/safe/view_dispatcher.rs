use super::{gui::Gui, submenu::Submenu, View, Widget};
use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{ffi::c_uchar, marker::PhantomData, ptr::NonNull};
use flipperzero::furi::sync::Mutex;
use flipperzero_sys as sys;

pub struct ViewDispatcher {
    pub(super) data: NonNull<sys::ViewDispatcher>,

    pub(super) gui: Gui,

    pub(super) views: Vec<(u32, Arc<View>)>,
    pub(super) submenus: Vec<(u32, Submenu)>,
    pub(super) widgets: Vec<(u32, Arc<Mutex<Widget>>)>,
}

#[repr(u8)]
pub enum ViewDispatcherType {
    Window = sys::ViewDispatcherType_ViewDispatcherTypeWindow,
    Desktop = sys::ViewDispatcherType_ViewDispatcherTypeDesktop,
    Fullscreen = sys::ViewDispatcherType_ViewDispatcherTypeFullscreen,
}

impl ViewDispatcher {
    pub fn new(gui: Gui, kind: ViewDispatcherType) -> Self {
        unsafe {
            let view_dispatcher = Self {
                data: NonNull::new_unchecked(sys::view_dispatcher_alloc()),

                gui,

                views: Vec::new(),
                submenus: Vec::new(),
                widgets: Vec::new(),
            };

            sys::view_dispatcher_attach_to_gui(
                view_dispatcher.data.as_ptr(),
                view_dispatcher.gui.data.as_ptr(),
                kind as c_uchar,
            );

            view_dispatcher
        }
    }

    pub fn view_switcher(&self) -> ViewSwitcher {
        ViewSwitcher {
            view_dispatcher_ptr: self.data.as_ptr(),
        }
    }

    pub fn add_view(&mut self, view: View, view_id: u32) -> Weak<View> {
        unsafe {
            sys::view_dispatcher_add_view(self.data.as_ptr(), view_id, view.data.as_ptr());
        }
        let view = Arc::new(view);
        let weak = Arc::downgrade(&view);
        self.views.push((view_id, view));
        weak
    }

    pub fn add_submenu(&mut self, submenu: Submenu, view_id: u32) {
        unsafe {
            sys::view_dispatcher_add_view(
                self.data.as_ptr(),
                view_id,
                submenu.as_view().data.as_ptr(),
            );
        }
        self.submenus.push((view_id, submenu));
    }

    pub fn add_widget_mutex(&mut self, widget: Widget, view_id: u32) -> Weak<Mutex<Widget>> {
        unsafe {
            sys::view_dispatcher_add_view(
                self.data.as_ptr(),
                view_id,
                widget.as_view().data.as_ptr(),
            );
        }
        let widget = Arc::new(Mutex::new(widget));
        let weak = Arc::downgrade(&widget);
        self.widgets.push((view_id, widget));
        weak
    }

    pub fn switch_to_view(&mut self, view_id: u32) {
        unsafe {
            sys::view_dispatcher_switch_to_view(self.data.as_ptr(), view_id);
        }
    }

    pub fn run(&mut self) {
        unsafe {
            sys::view_dispatcher_run(self.data.as_ptr());
        }
    }

    pub fn stop(&self) {
        unsafe {
            sys::view_dispatcher_stop(self.data.as_ptr());
        }
    }
}

impl Drop for ViewDispatcher {
    fn drop(&mut self) {
        unsafe {
            for view_id in self
                .views
                .iter()
                .map(|(view_id, _)| view_id)
                .chain(self.submenus.iter().map(|(view_id, _)| view_id))
                .chain(self.widgets.iter().map(|(view_id, _)| view_id))
                .copied()
            {
                sys::view_dispatcher_remove_view(self.data.as_ptr(), view_id);
            }

            sys::view_dispatcher_free(self.data.as_ptr());
        }
    }
}

// not safest implementation but eh
pub struct ViewSwitcher {
    pub(super) view_dispatcher_ptr: *mut sys::ViewDispatcher,
}

impl ViewSwitcher {
    pub fn switch_to_view(&self, view_id: u32) {
        unsafe {
            sys::view_dispatcher_switch_to_view(self.view_dispatcher_ptr, view_id);
        }
    }
}

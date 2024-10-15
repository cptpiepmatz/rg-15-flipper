use super::{gui::Gui, submenu::Submenu, View};
use alloc::vec::Vec;
use core::{ffi::c_uchar, ptr::NonNull};
use flipperzero_sys as sys;

pub struct ViewDispatcher {
    pub(super) data: NonNull<sys::ViewDispatcher>,

    pub(super) gui: Gui,

    pub(super) view_counter: u32,
    pub(super) views: Vec<(u32, View)>,
    pub(super) submenus: Vec<(u32, Submenu)>,
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

                view_counter: 0,
                views: Vec::new(),
                submenus: Vec::new(),
            };

            sys::view_dispatcher_attach_to_gui(
                view_dispatcher.data.as_ptr(),
                view_dispatcher.gui.data.as_ptr(),
                kind as c_uchar,
            );

            view_dispatcher
        }
    }

    pub fn add_view(&mut self, view: View) {
        let view_id = self.view_counter;

        unsafe {
            sys::view_dispatcher_add_view(self.data.as_ptr(), view_id, view.data.as_ptr());
        }

        self.view_counter += 1;
        self.views.push((view_id, view));
    }

    pub fn add_submenu(&mut self, submenu: Submenu) {
        let view_id = self.view_counter;

        unsafe {
            let view = sys::submenu_get_view(submenu.data.as_ptr());
            sys::view_dispatcher_add_view(self.data.as_ptr(), view_id, view);
        }

        self.view_counter += 1;
        self.submenus.push((view_id, submenu));
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
                .copied()
            {
                sys::view_dispatcher_remove_view(self.data.as_ptr(), view_id);
            }

            sys::view_dispatcher_free(self.data.as_ptr());
        }
    }
}

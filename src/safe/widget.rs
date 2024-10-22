use core::{ffi::CStr, mem::ManuallyDrop, ptr::NonNull};

use flipperzero_sys as sys;

use super::View;

pub struct Widget {
    pub(super) data: NonNull<sys::Widget>,
    view: ManuallyDrop<View>,
}

unsafe impl Send for Widget {}
unsafe impl Sync for Widget {}

impl Widget {
    pub fn new() -> Self {
        unsafe {
            let data = sys::widget_alloc();
            let view = sys::widget_get_view(data);
            let view = View {
                data: NonNull::new_unchecked(view),
                context: None,
                has_model: false,
            };
            let view = ManuallyDrop::new(view);
            let data = NonNull::new_unchecked(data);
            Self { data, view }
        }
    }

    pub fn reset(&mut self) {
        unsafe {
            sys::widget_reset(self.data.as_ptr());
        }
    }

    pub fn add_text_scroll_element(&mut self, x: u8, y: u8, width: u8, height: u8, text: &CStr) {
        unsafe {
            sys::widget_add_text_scroll_element(
                self.data.as_ptr(),
                x,
                y,
                width,
                height,
                text.as_ptr(),
            );
        }
    }

    pub fn as_view(&self) -> &View {
        &self.view
    }

    pub fn as_mut_view(&mut self) -> &mut View {
        &mut self.view
    }
}

impl Drop for Widget {
    fn drop(&mut self) {
        unsafe {
            sys::widget_free(self.data.as_ptr());
        }
    }
}

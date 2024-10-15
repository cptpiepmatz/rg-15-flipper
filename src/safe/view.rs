use core::ptr::NonNull;
use flipperzero_sys as sys;

pub struct View {
    pub(super) data: NonNull<sys::View>,
}

impl View {
    pub fn new() -> Self {
        unsafe {
            Self {
                data: NonNull::new_unchecked(sys::view_alloc()),
            }
        }
    }
}

impl Drop for View {
    fn drop(&mut self) {
        unsafe {
            sys::view_free(self.data.as_ptr());
        }
    }
}

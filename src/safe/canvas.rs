use core::{ffi::CStr, ptr::NonNull};

use flipperzero_sys as sys;

pub struct Canvas {
    pub(super) data: NonNull<sys::Canvas>,
}

impl Canvas {
    pub fn elements_button_left(&mut self, s: &CStr) {
        unsafe {
            sys::elements_button_left(self.data.as_ptr(), s.as_ptr());
        }
    }

    pub fn elements_button_right(&mut self, s: &CStr) {
        unsafe {
            sys::elements_button_right(self.data.as_ptr(), s.as_ptr());
        }
    }

    pub fn draw_dot(&mut self, x: i32, y: i32) {
        unsafe {
            sys::canvas_draw_dot(self.data.as_ptr(), x, y);
        }
    }

    pub fn draw_rbox(&mut self, x: i32, y: i32, width: usize, height: usize, radius: usize) {
        unsafe {
            sys::canvas_draw_rbox(self.data.as_ptr(), x, y, width, height, radius);
        }
    }

    pub fn invert_color(&mut self) {
        unsafe {
            sys::canvas_invert_color(self.data.as_ptr());
        }
    }

    pub fn string_width(&self, s: &CStr) -> u16 {
        unsafe { sys::canvas_string_width(self.data.as_ptr(), s.as_ptr()) }
    }

    pub fn draw_str(&mut self, x: i32, y: i32, s: &CStr) {
        unsafe {
            sys::canvas_draw_str(self.data.as_ptr(), x, y, s.as_ptr());
        }
    }
}

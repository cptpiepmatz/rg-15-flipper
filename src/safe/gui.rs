use flipperzero_sys as sys;
use flipperzero_sys::furi::UnsafeRecord;

pub struct Gui {
    pub(super) data: UnsafeRecord<sys::Gui>,
}

impl Gui {
    pub fn open() -> Self {
        unsafe {
            Self {
                data: UnsafeRecord::open(c"gui".as_ptr()),
            }
        }
    }
}

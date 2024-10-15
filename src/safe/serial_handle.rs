use core::{ffi::c_uchar, ptr::NonNull};
use flipperzero_sys as sys;

pub struct SerialHandle {
    pub(super) data: NonNull<sys::FuriHalSerialHandle>,
}

#[repr(u8)]
pub enum SerialId {
    Max = sys::FuriHalSerialId_FuriHalSerialIdMax,
    Usart = sys::FuriHalSerialId_FuriHalSerialIdUsart,
    Lpuart = sys::FuriHalSerialId_FuriHalSerialIdLpuart,
}

impl SerialHandle {
    pub fn acquire(serial_id: SerialId) -> Option<Self> {
        unsafe {
            let data = sys::furi_hal_serial_control_acquire(serial_id as c_uchar);
            let data = NonNull::new(data)?;
            Some(Self { data })
        }
    }
}

impl Drop for SerialHandle {
    fn drop(&mut self) {
        unsafe {
            sys::furi_hal_serial_control_release(self.data.as_ptr());
        }
    }
}

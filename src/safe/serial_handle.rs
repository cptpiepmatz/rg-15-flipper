use alloc::sync::Arc;
use core::{
    ffi::{c_uchar, c_void},
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    num::NonZeroUsize,
    ptr::{self, NonNull},
};
use flipperzero::{furi, println};
use flipperzero_sys as sys;

pub struct SerialHandle<M> {
    pub(super) data: NonNull<sys::FuriHalSerialHandle>,
    pub(super) context: Option<Arc<furi::stream_buffer::Sender>>,
    _phantom: PhantomData<M>,
}

pub mod serial_marker {
    pub struct Uninitialized;
    pub struct Initialized;
    pub struct Interrupted;
}

#[allow(unused)] // for completeness we have all variants
#[repr(u8)]
pub enum SerialId {
    Max = sys::FuriHalSerialId_FuriHalSerialIdMax,
    Usart = sys::FuriHalSerialId_FuriHalSerialIdUsart,
    Lpuart = sys::FuriHalSerialId_FuriHalSerialIdLpuart,
}

impl SerialHandle<serial_marker::Uninitialized> {
    pub fn acquire(serial_id: SerialId) -> Option<Self> {
        unsafe {
            let data = sys::furi_hal_serial_control_acquire(serial_id as c_uchar);
            let data = NonNull::new(data)?;
            Some(Self {
                data,
                context: None,
                _phantom: PhantomData,
            })
        }
    }

    pub fn init(self, baud: u32) -> SerialHandle<serial_marker::Initialized> {
        unsafe {
            sys::furi_hal_serial_init(self.data.as_ptr(), baud);
            mem::transmute(self)
        }
    }
}

impl SerialHandle<serial_marker::Initialized> {
    pub fn async_rx_start(&mut self, report_errors: bool) -> furi::stream_buffer::Receiver {
        let size = NonZeroUsize::new(4096).expect("non-zero value");
        let stream_buffer = furi::stream_buffer::StreamBuffer::new(size, 1);
        let (tx, rx) = stream_buffer.into_stream();

        let context = Arc::new(tx);
        let context_ptr = Arc::as_ptr(&context).cast_mut().cast();
        self.context = Some(context);

        unsafe {
            sys::furi_hal_serial_async_rx_start(
                self.data.as_ptr(),
                Some(raw_callback),
                context_ptr,
                report_errors,
            );
        }

        rx
    }

    pub fn tx(&self, buffer: &[u8]) {
        unsafe {
            sys::furi_hal_serial_tx(self.data.as_ptr(), buffer.as_ptr(), buffer.len());
            sys::furi_hal_serial_tx_wait_complete(self.data.as_ptr());
        }
    }

    // should be &mut self, but I don't want to sync that up correctly now
    pub fn set_br(&self, baud_rate: u32) {
        unsafe {
            sys::furi_hal_serial_set_br(self.data.as_ptr(), baud_rate);
        }
    }
}

impl SerialHandle<serial_marker::Interrupted> {
    #[allow(unused)]
    pub fn rx_available(&self) -> bool {
        unsafe { sys::furi_hal_serial_async_rx_available(self.data.as_ptr()) }
    }

    pub fn rx(&self) -> u8 {
        unsafe { sys::furi_hal_serial_async_rx(self.data.as_ptr()) }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SerialRxEvent {
    Data = sys::FuriHalSerialRxEvent_FuriHalSerialRxEventData,
    Idle = sys::FuriHalSerialRxEvent_FuriHalSerialRxEventIdle,
    FrameError = sys::FuriHalSerialRxEvent_FuriHalSerialRxEventFrameError,
    NoiseError = sys::FuriHalSerialRxEvent_FuriHalSerialRxEventNoiseError,
    OverrunError = sys::FuriHalSerialRxEvent_FuriHalSerialRxEventOverrunError,
}

unsafe extern "C" fn raw_callback(
    handle: *mut sys::FuriHalSerialHandle,
    event: sys::FuriHalSerialRxEvent,
    context: *mut c_void,
) {
    let mut event_copy: u8 = 0;
    ptr::write_volatile(&mut event_copy as *mut u8, event);

    let handle = NonNull::new_unchecked(handle);
    let handle = SerialHandle::<serial_marker::Interrupted> {
        data: handle,
        context: None,
        _phantom: PhantomData,
    };
    let handle = ManuallyDrop::new(handle);

    let event = match event {
        1 => SerialRxEvent::Data,
        2 => SerialRxEvent::Idle,
        4 => SerialRxEvent::FrameError,
        8 => SerialRxEvent::NoiseError,
        16 => SerialRxEvent::OverrunError,
        _ => unreachable!("enum is defined by these values only"),
    };

    let context: *const furi::stream_buffer::Sender = context.cast_const().cast();
    let context: &furi::stream_buffer::Sender = context.as_ref_unchecked();

    callback(&handle, event, context);
}

#[inline]
fn callback(
    handle: &SerialHandle<serial_marker::Interrupted>,
    event: SerialRxEvent,
    sender: &furi::stream_buffer::Sender,
) {
    if event != SerialRxEvent::Data {
        return;
    }
    let data = handle.rx();
    sender.send(&[data]);
}

impl<M> Drop for SerialHandle<M> {
    fn drop(&mut self) {
        println!("dropping serial handle");
        unsafe {
            sys::furi_hal_serial_async_rx_stop(self.data.as_ptr());
            sys::furi_hal_serial_deinit(self.data.as_ptr());
            sys::furi_hal_serial_control_release(self.data.as_ptr());
        }
    }
}

use alloc::{collections::BTreeMap, sync::Arc};
use core::{
    ffi::{c_void, CStr},
    ptr::{self, NonNull},
};
use flipperzero_sys as sys;

pub struct Submenu {
    pub(super) data: NonNull<sys::Submenu>,
}

impl Submenu {
    pub fn add_item<'l, I, C>(&'l mut self, label: &'l CStr, index: u32, context: Option<Arc<C>>)
    where
        I: SubmenuItem<Context = C>,
    {
        let submenu = self.data.as_ptr();
        let label = label.as_ptr();
        unsafe {
            match context {
                Some(context) => sys::submenu_add_item(
                    submenu,
                    label,
                    index,
                    Some(I::__select),
                    Arc::into_raw(context).cast_mut().cast(),
                ),
                None => sys::submenu_add_item(submenu, label, index, None, ptr::null_mut()),
            }
        };
    }
}

pub trait SubmenuItem {
    type Context;

    fn select(context: &Self::Context, index: u32);

    #[doc(hidden)]
    #[no_mangle]
    unsafe extern "C" fn __select(context: *mut c_void, index: u32) {
        let context: *const Self::Context = context.cast_const().cast();
        let context: &Self::Context = context.as_ref_unchecked();
        Self::select(context, index);
    }
}

impl Drop for Submenu {
    fn drop(&mut self) {
        unsafe {
            sys::submenu_free(self.data.as_ptr());
        }
    }
}

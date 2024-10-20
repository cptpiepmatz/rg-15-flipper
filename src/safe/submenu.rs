use alloc::{collections::BTreeMap, sync::Arc};
use core::{
    ffi::{c_void, CStr},
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};
use flipperzero_sys as sys;

use super::View;

pub struct Submenu {
    pub(super) data: NonNull<sys::Submenu>,
    // this view is purely a reference to the submenu here, so don't try to drop it
    view: ManuallyDrop<View>,
}

impl Submenu {
    pub fn new() -> Self {
        unsafe {
            let data = sys::submenu_alloc();
            let view = sys::submenu_get_view(data);
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

    pub fn as_view(&self) -> &View {
        &self.view
    }

    pub fn as_mut_view(&mut self) -> &mut View {
        &mut self.view
    }
}

pub trait SubmenuItem {
    type Context;

    fn select(context: &Self::Context, index: u32);

    #[doc(hidden)]
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

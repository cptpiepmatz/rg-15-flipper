use alloc::sync::Arc;
use core::{
    any::Any,
    ffi::c_void,
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};
use flipperzero_sys as sys;

use super::Canvas;

pub struct View {
    pub(super) data: NonNull<sys::View>,
    pub(super) context: Option<Arc<dyn Any>>,
    pub(super) has_model: bool,
}

unsafe impl Send for View {}
unsafe impl Sync for View {}

impl View {
    pub fn new() -> Self {
        unsafe {
            Self {
                data: NonNull::new_unchecked(sys::view_alloc()),
                context: None,
                has_model: false,
            }
        }
    }

    pub fn create_model<M: Any + Default + Sized>(&mut self) -> ModelGuard<M> {
        self.has_model = true;

        let model_type = sys::ViewModelType_ViewModelTypeLocking;
        unsafe {
            sys::view_allocate_model(self.data.as_ptr(), model_type, size_of::<M>());
            let model = sys::view_get_model(self.data.as_ptr());
            let model: *mut MaybeUninit<M> = model.cast();
            let model: &mut MaybeUninit<M> = model.as_mut_unchecked();
            let model = model.write(M::default());
            ModelGuard {
                view: &self.data,
                model,
            }
        }
    }

    pub fn get_model<M: Any>(&self) -> Option<ModelGuard<M>> {
        if !self.has_model {
            return None;
        }

        unsafe {
            let model = sys::view_get_model(self.data.as_ptr());
            let model: *mut dyn Any = model.cast::<M>();
            let model: &mut dyn Any = model.as_mut_unchecked();
            let model = model.downcast_mut()?;
            Some(ModelGuard {
                view: &self.data,
                model,
            })
        }
    }

    pub fn set_context(&mut self, context: Arc<dyn Any>) {
        unsafe {
            sys::view_set_context(
                self.data.as_ptr(),
                Arc::as_ptr(&context).cast::<c_void>().cast_mut(),
            );
        }

        self.context = Some(context);
    }

    pub fn set_previous_callback<C: ViewNavigationCallback>(&mut self) {
        unsafe {
            sys::view_set_previous_callback(self.data.as_ptr(), Some(C::__callback));
        }
    }

    pub fn set_draw_callback<C: ViewDrawCallback>(&mut self) {
        unsafe {
            sys::view_set_draw_callback(self.data.as_ptr(), Some(C::__callback));
        }
    }

    pub fn set_input_callback<C: ViewInputCallback>(&mut self) {
        unsafe {
            sys::view_set_input_callback(self.data.as_ptr(), Some(C::__callback));
        }
    }
}

pub struct ModelGuard<'m, M> {
    pub(super) view: &'m NonNull<sys::View>,
    pub model: &'m mut M,
}

impl<M> Drop for ModelGuard<'_, M> {
    fn drop(&mut self) {
        unsafe {
            sys::view_commit_model(self.view.as_ptr(), true);
        }
    }
}

pub trait ViewNavigationCallback {
    type Context: Any;

    fn callback(context: Option<&Self::Context>) -> u32;

    #[doc(hidden)]
    unsafe extern "C" fn __callback(context: *mut c_void) -> u32 {
        let context: *const dyn Any = context.cast_const().cast::<Self::Context>();
        let context: Option<&dyn Any> = context.as_ref();
        let context: Option<&Self::Context> = context.map(|any| any.downcast_ref()).flatten();
        Self::callback(context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InputKey {
    Up = sys::InputKey_InputKeyUp,
    Down = sys::InputKey_InputKeyDown,
    Right = sys::InputKey_InputKeyRight,
    Left = sys::InputKey_InputKeyLeft,
    Ok = sys::InputKey_InputKeyOk,
    Back = sys::InputKey_InputKeyBack,
}
pub trait ViewInputCallback {
    type Context: Any;

    fn callback(input_key: InputKey, context: Option<&Self::Context>) -> bool;

    #[doc(hidden)]
    unsafe extern "C" fn __callback(event: *mut sys::InputEvent, context: *mut c_void) -> bool {
        let context: *const dyn Any = context.cast_const().cast::<Self::Context>();
        let context: Option<&dyn Any> = context.as_ref();
        let context: Option<&Self::Context> = context.map(|any| any.downcast_ref()).flatten();

        let event: &mut sys::InputEvent = event.as_mut_unchecked();
        let input_key = match event.key {
            0 => InputKey::Up,
            1 => InputKey::Down,
            2 => InputKey::Right,
            3 => InputKey::Left,
            4 => InputKey::Ok,
            5 => InputKey::Back,
            _ => unreachable!(),
        };

        Self::callback(input_key, context)
    }
}

pub trait ViewDrawCallback {
    type Model: Any;

    fn callback(canvas: &mut Canvas, model: Option<&Self::Model>);

    #[doc(hidden)]
    unsafe extern "C" fn __callback(canvas: *mut sys::Canvas, model: *mut c_void) {
        let canvas = Canvas {
            data: NonNull::new_unchecked(canvas),
        };
        let mut canvas = ManuallyDrop::new(canvas);

        let model: *const dyn Any = model.cast_const().cast::<Self::Model>();
        let model: Option<&dyn Any> = model.as_ref();
        let model: Option<&Self::Model> = model.map(|any| any.downcast_ref()).flatten();

        Self::callback(&mut canvas, model);
    }
}

impl Drop for View {
    fn drop(&mut self) {
        unsafe {
            if self.has_model {
                sys::view_free_model(self.data.as_ptr());
            }

            sys::view_free(self.data.as_ptr());
        }
    }
}

pub struct ViewNone;

impl ViewNavigationCallback for ViewNone {
    type Context = ();

    fn callback(_: Option<&Self::Context>) -> u32 {
        const VIEW_NONE: u32 = 0xFFFFFFFF;
        VIEW_NONE
    }
}

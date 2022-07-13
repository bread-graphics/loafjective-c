// BSL 1.0 License

use crate::{class::Class, sel::Sel, Object, Result};
use __private::Sealed;
use core::{any::Any, mem, ptr::null_mut};

cfg_if::cfg_if! {
    if #[cfg(target_vendor = "apple")] {
        #[path = "apple.rs"]
        mod platform;
    } else {
        #[path = "gnustep.rs"]
        mod platform;
    }
}

/// An object that a message can be sent to.
///
/// # Safety
///
/// This should only ever be either an object, a class, or a block.
pub unsafe trait MessageTarget: Copy {
    /// Get the inner pointer.
    fn ptr(self) -> *mut ();
}

macro_rules! message_target_opaque {
    ($($name: ty)*) => {
        $(
            unsafe impl MessageTarget for $name {
                fn ptr(self) -> *mut () {
                    self.as_ptr() as _
                }
            }
        )*
    }
}

message_target_opaque! {
    Object Class
}

unsafe impl MessageTarget for Option<Object> {
    fn ptr(self) -> *mut () {
        self.map_or(null_mut(), |obj| obj.ptr())
    }
}

unsafe impl MessageTarget for &Superclass {
    fn ptr(self) -> *mut () {
        let p: *const () = self as *const Superclass as *const ();
        p as *mut ()
    }
}

/// Arguments that can be passed into a message.
///
/// # Safety
///
/// This should only really be implemented for tuple types. It has
/// a `Sealed` marker for this reason.
pub unsafe trait MessageArguments: Sealed + Sized {
    /// Call the given messaging function with this as an argument.
    ///
    /// # Safety
    ///
    /// The parameters must be valid for the given message.
    unsafe fn call_message<Target: MessageTarget, Return: Any>(
        self,
        function_ptr: unsafe extern "C" fn(),
        target: Target,
        sel: Sel,
    ) -> Return;
}

macro_rules! message_arguments {
    (@impl $($ident: ident)* | $($tt: tt)*) => {
        impl<$($ident),*> Sealed for ($($ident),* $($tt)*) {}

        #[allow(non_snake_case, unused_parens)]
        unsafe impl<$($ident),*> MessageArguments for ($($ident),* $($tt)*) {
            #[inline]
            unsafe fn call_message<
                Target: MessageTarget,
                Return: Any,
            >(
                self,
                function_ptr: unsafe extern "C" fn(),
                target: Target,
                sel: Sel,
            ) -> Return {
                // transmute the ptr to a function pointer
                let function_ptr: unsafe extern "C" fn(*const (), *const (), $($ident),*) -> Return =
                    mem::transmute(function_ptr);

                // disassemble the tuple and call the function
                let ($($ident),* $($tt)*) = self;
                function_ptr(target.ptr(), sel.as_ptr(), $($ident),*)
            }
        }
    };
    () => {
        message_arguments! { @impl | }
    };
    ($head: ident $($ident: ident)*) => {
        message_arguments! { @impl $head $($ident)* | , }
        message_arguments! { $($ident)* }
    }
}

message_arguments! {
    A B C D E F G H I J K L M N O P Q R S T U V W X Y Z
}

/// Try to send a message.
///
/// # Safety
///
/// The message must be valid for the given target.
pub unsafe fn send_message<Target: MessageTarget, Return: Any, Arguments: MessageArguments>(
    target: Target,
    sel: Sel,
    arguments: Arguments,
    checked: bool,
) -> Result<Return> {
    let fn_ptr = platform::send_message_function::<Return>(target.ptr(), sel);

    if checked || cfg!(debug_assertions) {
        crate::error::error_catcher(move || arguments.call_message(fn_ptr, target, sel))
    } else {
        Ok(arguments.call_message(fn_ptr, target, sel))
    }
}

/// Send a message to the object's superclass.
///
/// # Safety
///
/// The message must be valid for the given target.
pub unsafe fn send_super_message<
    Target: MessageTarget,
    Return: Any,
    Arguments: MessageArguments,
>(
    target: Target,
    superclass: Class,
    sel: Sel,
    arguments: Arguments,
    checked: bool,
) -> Result<Return> {
    let superclass = Superclass {
        receiver: target.ptr(),
        superclass,
    };
    let fn_ptr = platform::send_super_message_function::<Return>(&superclass, sel);

    if checked || cfg!(debug_assertions) {
        crate::error::error_catcher(move || arguments.call_message(fn_ptr, &superclass, sel))
    } else {
        Ok(arguments.call_message(fn_ptr, &superclass, sel))
    }
}

/// Representation of the super-class.
#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct Superclass {
    pub receiver: *const (),
    pub superclass: Class,
}

mod __private {
    #[doc(hidden)]
    pub trait Sealed {
        fn __sealed_marker() {}
    }
}

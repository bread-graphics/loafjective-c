// BSL 1.0 License

use crate::{cstr::{c_char, CStr}, Object};
use core::{
    fmt,
    hint::unreachable_unchecked,
    mem::{ManuallyDrop, MaybeUninit},
};

/// Represents an error that can be emitted by the Objective C code.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Error {
    id: Object,
}

impl Error {
    /// Get the underlying pointer backing this `Error`.
    pub fn as_ptr(&self) -> *const () {
        self.id.as_ptr()
    }

    /// Create a new `Error` from an underlying pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be non-null and a valid pointer to the Objective C
    /// object.
    pub unsafe fn from_ptr(ptr: *const ()) -> Error {
        Error {
            id: Object::from_ptr(ptr),
        }
    }
}

fn write_nsstring(nsstr: Object, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // get the UTF-8 encoded string
    if let Ok(encoded_str) = unsafe { msg_send![*const c_char => nsstr, UTF8String] } {
        // convert to a rust string
        let encoded_str = unsafe { CStr::from_ptr(encoded_str) };

        match encoded_str.to_str() {
            Ok(str) => f.write_str(str),
            Err(_) => fmt::Debug::fmt(encoded_str, f),
        }
    } else {
        f.write_str("<failed to get error information>")
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        Error {
            id: unsafe { msg_send![self.id, retain] }.expect("failed to retain error"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct PrintClassName<'a>(&'a Error);

        impl<'a> fmt::Debug for PrintClassName<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if let Ok(class_name) = unsafe { msg_send![self.0.id, className] } {
                    write_nsstring(class_name, f)
                } else {
                    f.write_str("<failed to get class name>")
                }
            }
        }

        f.debug_tuple("Error").field(&PrintClassName(self)).finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // try to load the classes we need
        if let Some(ns_exception) = optional_class!(NSException) {
            if let Ok(true) = unsafe { msg_send![self.id, isKindOfClass: ns_exception] } {
                if let Ok(reason) = unsafe { msg_send![self.id, reason] } {
                    return write_nsstring(reason, f);
                }

                return f
                    .write_str("<an error occurred while accessing the exception's properties>");
            }
        }

        f.write_str("Objective-C threw an exception that was not of type NSException")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl Drop for Error {
    fn drop(&mut self) {
        // destroy the ID
        let _ = unsafe { msg_send![() => self.id, release] };
    }
}

/// Try to run a function that may cause an Objective C exception
/// to be raised.
///
/// # Safety
///
/// The inner function must never panic. If it panics, undefined
/// behavior will occur.
pub(crate) unsafe fn error_catcher<R, F: FnOnce() -> R>(f: F) -> Result<R> {
    // see loafTryRunAndCatch in the error.m file for more information
    // on the implementation of this function.

    /// A structure that binds together the closure and its result.
    struct ClosureExecution<R, F> {
        function: ManuallyDrop<F>,
        result: MaybeUninit<R>,
    }

    unsafe extern "C" fn run_the_closure<R, F: FnOnce() -> R>(closure: *mut ()) {
        let closure = &mut *(closure as *mut ClosureExecution<R, F>);

        // read the closure out of the pointer
        // SAFETY: closure is in a ManuallyDrop so this is safe
        let function: F = ManuallyDrop::take(&mut closure.function);
        let result = function();

        // we use the same memory to hold the closure and the return value,
        // so write the return value back into the closure
        closure.result = MaybeUninit::new(result);
    }

    // allocate the stack memory for the closure
    let mut closure = ClosureExecution::<R, F> {
        function: ManuallyDrop::new(f),
        result: MaybeUninit::uninit(),
    };
    let closure_ptr = &mut closure as *mut ClosureExecution<R, F>;
    let mut error = MaybeUninit::<*const ()>::uninit();

    // call the function
    let return_value = loafTryRunAndCatch(
        Some(run_the_closure::<R, F>),
        closure_ptr as *mut (),
        error.as_mut_ptr(),
    );

    // see if it errored out
    match return_value {
        0 => Ok(closure.result.assume_init()),
        1 => Err(Error::from_ptr(error.assume_init())),
        _ => unreachable_unchecked(),
    }
}

/// Convenience type for a result.
pub type Result<T = ()> = core::result::Result<T, Error>;

extern "C" {
    fn loafTryRunAndCatch(
        function: Option<unsafe extern "C" fn(*mut ())>,
        closure: *mut (),
        error: *mut *const (),
    ) -> libc::c_int;
}
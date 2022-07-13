// BSL 1.0 License

//! Types for representing the `SEL` construct in Objective C.

use crate::{cstr::CStr, ffi};
use core::{fmt, ptr::NonNull, str};

opaque_type! {
    #[doc = "A selector to be used to select functions in Objective C."]
    Sel, AtomicSel
}

impl fmt::Debug for Sel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Sel").field(&self.name()).finish()
    }
}

impl Sel {
    /// Creates a new selector from a C string.
    pub fn new(name: &CStr) -> Sel {
        let ptr = unsafe { ffi::sel_registerName(name.as_ptr()) };
        Sel {
            ptr: NonNull::new(ptr as _).expect("SEL pointer should never be null"),
        }
    }

    /// Gets the name of the selector.
    pub fn name(&self) -> &str {
        let c_ptr = unsafe { ffi::sel_getName(self.ptr.as_ptr() as _) };
        let c_str = unsafe { CStr::from_ptr(c_ptr) };

        str::from_utf8(c_str.to_bytes()).expect("Selectors should be valid UTF-8")
    }
}

// BSL 1.0 License

use crate::{cstr::CStr, ffi};
use core::{fmt, ptr::NonNull, str};

opaque_type! {
    #[doc = "An Objective-C class."]
    Class, AtomicClass
}

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Class").field(&self.name()).finish()
    }
}

impl Class {
    /// Tries to get a class from a C string.
    pub fn new(name: &CStr) -> Option<Class> {
        let ptr = unsafe { ffi::objc_getClass(name.as_ptr()) };
        Some(Class {
            ptr: NonNull::new(ptr as _)?,
        })
    }

    /// Get the name of this class.
    pub fn name(&self) -> &str {
        let c_ptr = unsafe { ffi::class_getName(self.ptr.as_ptr()) };
        let c_str = unsafe { CStr::from_ptr(c_ptr) };

        str::from_utf8(c_str.to_bytes()).expect("Class names should be valid UTF-8")
    }

    /// Get the superclass for this class.
    pub fn superclass(&self) -> Class {
        let ptr = unsafe { ffi::class_getSuperclass(self.ptr.as_ptr()) };
        Class {
            ptr: NonNull::new(ptr as _).expect("Class pointer should never be null"),
        }
    }
}

// BSL 1.0 License

use crate::cstr::c_char;

#[link(name = "objc", kind = "dylib")]
extern "C" {
    pub fn sel_registerName(name: *const c_char) -> *const ();
    pub fn sel_getName(sel: *const ()) -> *const c_char;
    pub fn objc_getClass(name: *const c_char) -> *const ();
    pub fn class_getName(cls: *const ()) -> *const c_char;
    pub fn class_getSuperclass(cls: *const ()) -> *const ();
}

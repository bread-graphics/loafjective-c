// BSL 1.0 License

use super::Superclass;
use crate::Sel;
use core::any::Any;

#[link(name = "objc", kind = "dylib")]
extern "C" {
    fn objc_msg_lookup(receiver: *const (), op: *const ()) -> unsafe extern "C" fn();
    fn objc_msg_lookup_super(sup: *const Superclass, sel: *const ()) -> unsafe extern "C" fn();
}

pub(crate) unsafe fn send_message_function<R: Any>(
    receiver: *const (),
    sel: Sel,
) -> unsafe extern "C" fn() {
    objc_msg_lookup(receiver, sel.as_ptr())
}

pub(crate) unsafe fn send_super_message_function<R: Any>(
    receiver: &Superclass,
    sel: Sel,
) -> unsafe extern "C" fn() {
    objc_msg_lookup_super(receiver, sel.as_ptr())
}

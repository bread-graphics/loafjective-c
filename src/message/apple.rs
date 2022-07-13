// BSL 1.0 License

// Binding to Apple Objective C functionality vary from platform to platform.

mod arch {
    use core::{
        any::{Any, TypeId},
        mem,
    };

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "x86")] {
            #[link(name = "objc", kind = "dylib")]
            extern "C" {
                fn objc_msgSend();
                fn objc_msgSend_fpret();
                fn objc_msgSend_stret();
                fn objc_msgSendSuper();
                fn objc_msgSendSuper_stret();
            }

            pub(crate) fn msg_function<R: Any>() -> unsafe extern "C" fn() {
                let type_id = TypeId::of::<R>();
                let size = mem::size_of::<R>();

                // floats are returned through fpret
                // non-primitives are returned through stret
                if type_id == TypeId::of::<f32>() || type_id == TypeId::of::<f64>() {
                    objc_msgSend_fpret
                } else if &[0, 1, 2, 4, 8].contains(&size) {
                    objc_msgSend
                } else {
                    objc_msgSend_stret
                }
            }

            pub(crate) fn super_function<R: Any>() -> unsafe extern "C" fn() {
                if &[0, 1, 2, 4, 8].contains(&mem::size_of::<R>()) {
                    objc_msgSendSuper
                } else {
                    objc_msgSendSuper_stret
                }
            }
        } else if #[cfg(target_arch = "x86_64")] {
            #[link(name = "objc", kind = "dylib")]
            extern "C" {
                fn objc_msgSend();
                fn objc_msgSend_stret();
                fn objc_msgSendSuper();
                fn objc_msgSendSuper_stret();
            }

            /// Types more than two words in length use stret
            fn stret(sz: usize) -> bool {
                sz > 16
            }

            pub(crate) fn msg_function<R: Any>() -> unsafe extern "C" fn() {
                if stret(mem::size_of::<R>()) {
                    objc_msgSend_stret
                } else {
                    objc_msgSend
                }
            }

            pub(crate) fn super_function<R: Any>() -> unsafe extern "C" fn() {
                if stret(mem::size_of::<R>()) {
                    objc_msgSendSuper_stret
                } else {
                    objc_msgSendSuper
                }
            }
        } else if #[cfg(target_arch = "arm")] {
            #[link(name = "objc", kind = "dylib")]
            extern "C" {
                fn objc_msgSend();
                fn objc_msgSend_stret();
                fn objc_msgSendSuper();
                fn objc_msgSendSuper_stret();
            }

            /// All types more than a word in length except for
            /// double-word fundamentals should use stret.
            fn stret<R: Any>() -> bool {
                let type_id = TypeId::of::<R>();

                mem::size_of::<R>() > 4 &&
                    type_id != TypeId::of::<i64>() &&
                    type_id != TypeId::of::<u64>() &&
                    type_id != TypeId::of::<f64>()
            }

            pub(crate) fn msg_function<R: Any>() -> unsafe extern "C" fn() {
                if stret::<R>() {
                    objc_msgSend_stret
                } else {
                    objc_msgSend
                }
            }

            pub(crate) fn super_function<R: Any>() -> unsafe extern "C" fn() {
                if stret(mem::size_of::<R>()) {
                    objc_msgSendSuper_stret
                } else {
                    objc_msgSendSuper
                }
            }
        } else if #[cfg(target_arch = "aarch64")] {
            // there is no stret on aarch64

            #[link(name = "objc", kind = "dylib")]
            extern "C" {
                fn objc_msgSend();
                fn objc_msgSendSuper();
            }

            pub(crate) fn msg_function<R: Any>() -> unsafe extern "C" fn() {
                objc_msgSend
            }

            pub(crate) fn super_function<R: Any>() -> unsafe extern "C" fn() {
                objc_msgSendSuper
            }
        } else {
            pub(crate) fn msg_function<R: Any>() -> unsafe extern "C" fn() {
                panic!("Unsupported architecture")
            }

            pub(crate) fn super_function<R: Any>() -> unsafe extern "C" fn() {
                panic!("Unsupported architecture")
            }
        }
    }
}

pub(crate) unsafe fn send_message_function<R: Any>(
    _receiver: *const (),
    _sel: Sel,
) -> unsafe extern "C" fn() {
    arch::msg_function::<R>()
}

pub(crate) unsafe fn send_super_message_function<R: Any>(
    _receiver: &Superclass,
    _sel: Sel,
) -> unsafe extern "C" fn() {
    arch::super_function::<R>()
}

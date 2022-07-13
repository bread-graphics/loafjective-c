// BSL 1.0 License

/// Crate-internal macro for creating an opaque wrapper type.
macro_rules! opaque_type {
    ($(#[$meta: meta])* $ident: ident) => {
        $(#[$meta])*
        #[repr(C)]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $ident {
            ptr: core::ptr::NonNull<()>,
        }

        impl $ident {
            /// Get the pointer associated with this object.
            pub fn as_ptr(&self) -> *const () {
                self.ptr.as_ptr()
            }

            /// Create a selector from a raw pointer.
            ///
            /// # Safety
            ///
            /// The pointer must be non-null and a valid pointer to the
            /// Objective C object.
            pub unsafe fn from_ptr(ptr: *const ()) -> $ident {
                $ident {
                    ptr: core::ptr::NonNull::new_unchecked(ptr as _),
                }
            }
        }
    };
    ($(#[$meta: meta])* $ident: ident, $aident: ident) => {
        opaque_type!($(#[$meta])* $ident);

        /// Implementation detail.
        #[doc(hidden)]
        pub struct $aident {
            ptr: core::sync::atomic::AtomicPtr<()>,
        }

        impl $aident {
            /// Create a new, empty atomic holder.
            pub const fn new() -> Self {
                Self {
                    ptr: core::sync::atomic::AtomicPtr::new(core::ptr::null_mut()),
                }
            }

            /// Try to fetch the interior pointer.
            pub fn get_or_init(&self, init: impl FnOnce() -> $ident) -> $ident {
                use core::sync::atomic::Ordering::*;
                let mut ptr = self.ptr.load(Acquire);
                if ptr.is_null() {
                    let sel = init();
                    match self.ptr.compare_exchange(ptr, sel.as_ptr() as _, AcqRel, Acquire) {
                        Ok(_) => return sel,
                        Err(e) => { ptr = e; }
                    }
                }
                unsafe { $ident::from_ptr(ptr) }
            }

            /// Try to fetch the interior pointer, but can fail.
            pub fn try_get_or_init(&self, init: impl FnOnce() -> Option<$ident>)
                 -> Option<$ident> {
                use core::sync::atomic::Ordering::*;
                let mut ptr = self.ptr.load(Acquire);
                if ptr.is_null() {
                    let sel = init()?;
                    match self.ptr.compare_exchange(ptr, sel.as_ptr() as _, AcqRel, Acquire) {
                        Ok(_) => return Some(sel),
                        Err(e) => { ptr = e; }
                    }
                }
                Some(unsafe { $ident::from_ptr(ptr) })
            }
        }
    }
}

/// Generate a selector.
#[macro_export]
macro_rules! sel {
    ($name: ident) => {
        $crate::sel!(@raw_str stringify!($name))
    };
    ($($name: ident :)+) => {
        $crate::sel!(@raw_str concat!($(stringify!($name), ":")+,))
    };
    (@raw_str $name: expr) => {{
        static CACHED: $crate::__private::AtomicSel =
            $crate::__private::AtomicSel::new();
        CACHED.get_or_init(|| {
            let name = $crate::__private::cstr!($name);
            $crate::Sel::new(name)
        })
    }};
}

/// Fetch a class, optionally.
#[macro_export]
macro_rules! optional_class {
    ($name: ident) => {{
        static CACHED: $crate::__private::AtomicClass = $crate::__private::AtomicClass::new();
        CACHED.try_get_or_init(|| {
            let name = $crate::__private::cstr!(stringify!($name));
            $crate::Class::new(name)
        })
    }};
}

/// Fetch a class.
#[macro_export]
macro_rules! class {
    ($name: ident) => {{
        match $crate::optional_class($name) {
            Some(cls) => cls,
            None => {
                panic!("Could not find class `{}`", $name);
            }
        }
    }};
}

/// Send a message.
#[macro_export]
macro_rules! msg_send {
    (@make_fncall $ty: ty, $fname: ident, $obj: expr, $checked: expr, $name: ident) => {{
        $crate::$fname::<_, $ty, _>(
            $obj,
            $crate::sel!($name),
            (),
            $checked
        )
    }};
    (@make_fncall $ty: ty, $fname: ident, $obj: expr, $checked: expr, $($name: ident : $arg: expr)+) => {{
        $crate::$fname::<_, $ty, _>(
            $obj,
            $crate::sel!($($name:)+),
            ($($arg,)*),
            $checked
        )
    }};
    (@expl_checked $checked: expr, $ty: ty => $obj: expr, $($args: tt)*) => {
        $crate::msg_send!(
            @make_fncall $ty,
            send_message,
            $obj,
            $checked,
            $($args)*
        )
    };
    (@expl_checked $checked: expr, $ty: ty =>
        super($obj: expr, $sclass: expr), $($args: tt)*) => {
        $crate::msg_send!(
            @make_fncall $ty,
            send_super_message,
            $obj,
            $checked,
            $($args)*
        )
    };
    (@expl_checked $checked: expr, $($other: tt)*) => {
        $crate::msg_send!(
            @expl_checked $checked,
            _ => $($other)*
        )
    };
    ([unchecked] $($args: tt)*) => {{
        let value = $crate::msg_send!(@expl_checked false, $($args)*);
        value.expect("Objective-C threw an exception")
    }};
    ($($args: tt)*) => {
        $crate::msg_send!(@expl_checked true, $($args)*)
    };
}

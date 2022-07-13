//              Copyright John Nunley 2022
// Distributed under the Boost Software License, Version 1.0.
//       (See accompanying file LICENSE or copy at
//         https://www.boost.org/LICENSE_1_0.txt)

//! A library for interfacing with the Objective C runtime.
//!
//! The design of this crate is inspired by the [`objc`] crate, but
//! aims to fix several design deficiencies in the original crate.
//!
//! - The original crate, by default, does not handle Objective C
//!   objections. This means that exceptions will unwind into Rust code,
//!   which will lead to undefined behavior. A feature can be enabled
//!   to make these instances panic instead, but as of the time of
//!   writing this functionality is broken, and even so panicking
//!   is often not a desirable result. This crate fixes this issue
//!   by making the `msg_send!` macro return a `Result<T, Error>`
//!   instead of `T`.
//! - The original crate uses very poor macro customs. This crate
//!   fixes this by allowing macros to be used without importing
//!   all other macros from the crate.
//! - The original crate did not have a bread pun in its name.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
mod macros;

pub(crate) mod ffi;

mod class;
pub use class::Class;

pub(crate) mod cstr;

mod error;
pub use error::{Error, Result};

mod message;
pub use message::{send_message, send_super_message, MessageArguments, MessageTarget};

mod sel;
pub use sel::Sel;

opaque_type! {
    #[doc = "A pointer to an Objective C object."]
    Object
}

#[allow(non_camel_case_types)]
pub type id = Object;

/// Private types used in macros.
/// 
/// These types are not meant to be used directly, and are semver
/// exempt.
#[doc(hidden)]
pub mod __private {
    pub use crate::{class::AtomicClass, sel::AtomicSel};

    pub use cstr_core::cstr; 
}

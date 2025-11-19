//#![allow(clippy::missing_errors_doc)]


/// cbindgen:ignore
#[cfg(not(feature = "unsafe"))]
#[allow(non_camel_case_types, non_upper_case_globals,
    non_snake_case, unused, unsafe_op_in_unsafe_fn, 
    clippy::missing_safety_doc, clippy::unreadable_literal, 
    clippy::pub_underscore_fields, clippy::transmute_ptr_to_ptr)]
mod cbindings;

/// cbindgen:ignore
#[cfg(feature = "unsafe")]
#[allow(non_camel_case_types, non_upper_case_globals,
    non_snake_case, unused, unsafe_op_in_unsafe_fn, 
    clippy::missing_safety_doc, clippy::unreadable_literal, 
    clippy::pub_underscore_fields, clippy::transmute_ptr_to_ptr,
    clippy::must_use_candidate)]
pub mod cbindings;

#[cfg(feature = "safe")]
mod safe_api;

#[cfg(feature = "safe")]
pub use safe_api::*;

use crate::cbindings::CApiVersion;

pub mod misc;

pub mod c {
    pub use super::cbindings::CPluginInfo;
    pub use super::cbindings::CUuid;
    pub use super::cbindings::CApiVersion;
}

#[unsafe(no_mangle)]
#[used]
pub static API_VERSION: CApiVersion = CApiVersion::cargo();
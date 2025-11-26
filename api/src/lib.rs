#![no_std]

/// cbindgen:ignore
#[cfg(not(feature = "unsafe"))]
#[allow(non_camel_case_types, non_upper_case_globals,
    non_snake_case, unused, unsafe_op_in_unsafe_fn, 
    clippy::missing_safety_doc, clippy::unreadable_literal, 
    clippy::pub_underscore_fields, clippy::transmute_ptr_to_ptr,
    clippy::must_use_candidate, clippy::absolute_paths,
    clippy::arbitrary_source_item_ordering, clippy::use_self,
    clippy::missing_inline_in_public_items, clippy::renamed_function_params,
    clippy::decimal_literal_representation, clippy::exhaustive_structs,
    clippy::unseparated_literal_suffix, clippy::allow_attributes_without_reason,
    clippy::allow_attributes, clippy::indexing_slicing,
    clippy::exhaustive_enums
)]
mod cbindings;

/// cbindgen:ignore
#[cfg(feature = "unsafe")]
#[allow(non_camel_case_types, non_upper_case_globals,
    non_snake_case, unused, unsafe_op_in_unsafe_fn, 
    clippy::missing_safety_doc, clippy::unreadable_literal, 
    clippy::pub_underscore_fields, clippy::transmute_ptr_to_ptr,
    clippy::must_use_candidate, clippy::absolute_paths,
    clippy::arbitrary_source_item_ordering, clippy::use_self,
    clippy::missing_inline_in_public_items, clippy::renamed_function_params,
    clippy::decimal_literal_representation, clippy::exhaustive_structs,
    clippy::unseparated_literal_suffix, clippy::allow_attributes_without_reason,
    clippy::allow_attributes, clippy::indexing_slicing,
    clippy::exhaustive_enums
)]
pub mod cbindings;

#[cfg(feature = "safe")]
mod safe_api;

#[cfg(feature = "safe")]
pub use safe_api::*;

pub mod misc;

pub use cbindings::CPluginInfo;
pub use cbindings::CUuid;
pub use cbindings::CApiVersion as ApiVersion;

use crate::cbindings::CApiVersion;
#[unsafe(no_mangle)]
#[used]
pub static API_VERSION: CApiVersion = CApiVersion::cargo();
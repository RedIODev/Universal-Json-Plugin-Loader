use crate::capi::cbindings::destroyString;

pub mod api;
/// cbindgen:ignore
#[allow(non_camel_case_types, non_upper_case_globals, unused)]
pub mod cbindings;

// impl Drop for cbindings::String {
//     fn drop(&mut self) {
//         unsafe { destroyString(self) };
//     }
// }
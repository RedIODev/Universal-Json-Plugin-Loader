// use std::{error::Error, hash::Hash, str::Utf8Error};

// use derive_more::Display;
// use thiserror::Error;

// use crate::cbindings::{
//     CEventHandler, CEventHandlerFP, CRequestHandlerFP, CString, CUuid, CEndpointResponse,
//     CList_String, CServiceError, createListString, createString, destroyListString, destroyString,
//     getLengthString, getViewString, isValidListString, isValidString,
// };





// impl Drop for CList_String { //make compatible with string slices slices
//     fn drop(&mut self) {
//         unsafe { destroyListString(self) };
//     }
// }

// impl CList_String {
//     pub fn as_array(&self) -> Result<&[CString], InvalidList> {
//         if unsafe { !isValidListString(self) } {
//             return Err(InvalidList);
//         }
//         if self.data.is_null() {
//             return Ok(&[]);
//         }
//         let slice = unsafe { std::slice::from_raw_parts(self.data, self.length as usize) };
//         Ok(slice)
//     }
// }

// impl<T> From<T> for CList_String
// where
//     T: Into<Box<[CString]>>,
// {
//     fn from(value: T) -> Self {
//         let boxed_list: Box<[_]> = value.into();
//         let leaked = unsafe { &mut *Box::into_raw(boxed_list) };
//         let ptr = leaked.as_mut_ptr();
//         let length = leaked.len();
//         unsafe { createListString(ptr, length as u32, Some(drop_list_string)) }
//     }
// }

// unsafe extern "C" fn drop_list_string(list: *mut CString, length: u32) {
//     let slice = unsafe { std::slice::from_raw_parts_mut(list, length as usize) };
//     let _ = unsafe { Box::from_raw(slice) };
// }

// #[derive(Error, Display, Debug)]
// pub struct InvalidList;



// impl CUuid {
//     pub const fn as_u64_pair(&self) -> (u64, u64) {
//         (self.higher, self.lower)
//     }

//     pub const fn from_u64_pair(pair: (u64, u64)) -> Self {
//         Self {
//             higher: pair.0,
//             lower: pair.1,
//         }
//     }
// }

// impl Clone for CUuid {
//     fn clone(&self) -> Self {
//         *self
//     }
// }

// impl Copy for CUuid {}

// impl PartialEq for CUuid {
//     fn eq(&self, other: &Self) -> bool {
//         self.higher == other.higher && self.lower == other.lower
//     }
// }

// impl Hash for CUuid {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.higher.hash(state);
//         self.lower.hash(state);
//     }
// }

// impl Eq for CUuid {}

// pub trait OptionWrapped {
//     type Unwrapped;
// }

// impl<T> OptionWrapped for Option<T> {
//     type Unwrapped = T;
// }

// pub type EventHandlerFP = <CEventHandlerFP as OptionWrapped>::Unwrapped;

// pub type RequestHandlerFP = <CRequestHandlerFP as OptionWrapped>::Unwrapped;




// impl std::fmt::Display for CServiceError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }



extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use core::{str::{self, Utf8Error}, slice};
use core::ptr::NonNull;

use derive_more::Display;
use thiserror::Error;

use crate::{ServiceError, cbindings::{
    CApiVersion, CList_String, CString, CUuid, asErrorString, createListString, createString, destroyListString, destroyString, emptyListString, fromErrorString, getLengthString, getViewString, isValidListString, isValidString
}};

#[cfg(feature = "safe")]
impl From<CUuid> for uuid::Uuid {
    #[inline]
    fn from(value: CUuid) -> Self {
        Self::from_u64_pair(value.higher, value.lower)
    }
}

#[cfg(feature = "safe")]
impl From<uuid::Uuid> for CUuid {
    #[inline]
    fn from(value: uuid::Uuid) -> Self {
        let (higher, lower) = value.as_u64_pair();
        Self { higher, lower }
    }
}

impl Drop for CString {
    #[inline]
    fn drop(&mut self) {
        // SAFETY:
        // Calling dropString on an instance of CString is the correct way to dispose of 
        // an instance of this type according to the c-api.
        unsafe { destroyString(self); }
    }
}

impl CString {
    ///
    /// Tries to get a `&str` from this `CString`.
    /// # Errors
    /// The access might fail in case the `CString` instance is not valid according to [`isValidString`].
    /// Or the resulting string is not a valid UTF-8 string required by `str`.
    /// 
    #[inline]
    pub fn as_str(&self) -> Result<&str, ApiMiscError> {
        // SAFETY:
        // Calling isValidString is always safe.
        // Any bit pattern of CString is a valid argument for isValidString. 
        if unsafe { !isValidString(self) } {
            // SAFETY:
            // Calling asErrorString is always safe.
            // The function checks the validity of it's argument internally.
            // Returns null if the argument is not a valid error.
            let service_error_ptr = unsafe { asErrorString(self) };
            let service_error = NonNull::new(service_error_ptr).ok_or(ApiMiscError::InvalidString)?;
            // SAFETY:
            // CServiceErrors returned by asErrorString are always null or a valid error.
            // We checked for null by converting it to NonNull.
            unsafe { service_error.read() }.to_rust()?;
        }
        // SAFETY:
        // Calling getLengthString with a valid CString as checked above is safe.
        // The Length value can be trusted as we checked for an invalid string already.
        let len = unsafe { getLengthString(self) };
        // SAFETY:
        // Calling getViewString with a valid CString, 0 and it's reported length is safe 
        // as it get's a slice over the entire CString.
        // The owner stays the CString instance as getView only creates a non owning view.
        // The view is valid as long as the CString is valid which is upheld by the lifetime relationship created in the next lines.
        let ptr = unsafe { getViewString(self, 0, len) };
        // SAFETY:
        // Creating a &'a [u8] from a pointer returned from getViewString is valid because 
        // the lifetime of the view is equal to the lifetime of the CString which is bound to the instance of CString through the elided lifetime.
        Ok(str::from_utf8(unsafe {
            slice::from_raw_parts(ptr, len)
        })?)
    }
}

impl From<ServiceError> for CString {
    #[inline]
    fn from(value: ServiceError) -> Self {
        // SAFETY:
        // Creating an Error instance of CString from a valid CServiceError is almost always valid.
        // The only value not valid is CServiceError::Success. This variant can't be created from
        // an ServiceError instance.
        unsafe {fromErrorString(value.into())}
    }
}

impl<'string> From<&'string CString> for Result<&'string str, ServiceError> {
    #[inline]
    fn from(value: &'string CString) -> Self {
        value.as_str().map_err(|api_error| match api_error {
                ApiMiscError::Service(service_error) => service_error,
                ApiMiscError::Utf8(_)| ApiMiscError::InvalidString => ServiceError::InvalidString,
                ApiMiscError::InvalidList => unreachable!("as_str() can't produce this error.")
            }
        )
    }
}

impl From<CString> for Result<String, ServiceError> {
    #[inline]
    fn from(value: CString) -> Self {
        Result::<& str,_>::from(&value).map(String::from)
    }
}

///
/// A trait representing values that can be converted to a `CString`.
/// 
pub trait ToCString {
    ///
    /// Consumes this self value and turns it into a `CString`.
    /// 
    fn to_c_string(self) -> CString;
}

impl<T: Into<Box<str>>> ToCString for Result<T, ServiceError> {
    #[inline]
    fn to_c_string(self) -> CString {
        match self {
            Ok(str) => CString::from(str.into()),
            Err(error) => error.into()
        }
    }
}

impl<T: Into<Box<str>>> From<T> for CString {
    #[inline]
    fn from(value: T) -> Self {
        let boxed = value.into();
        let length = boxed.len();
        let leaked = Box::into_raw(boxed);
        let ptr = leaked.cast();
        // SAFETY:
        // Calling createString with a valid ptr, the corresponding length and drop function like we ensured here
        // creates a valid CString instance from createString.
        unsafe { createString(ptr, length, Some(drop_string)) }
    }
}

impl Drop for CList_String {
    #[inline]
    fn drop(&mut self) {
        // SAFETY:
        // Calling destroyListString on an instance of CList_String is the correct way to dispose of an instance
        // of this type. For invalid instances destroyListString is defined as a nop. Therefore its safe to call in any way.
        unsafe { destroyListString(self); }
    }
}

impl CList_String {
    ///
    /// Converts this String array into a Vec<&str>.
    /// 
    /// The Vec is allocated but the &str instances are still references to the `CList_String` instance.
    /// # Errors
    /// This operation might fail either because the List itself is invalid or a string inside it is.
    /// 
    #[inline]
    pub fn as_array(&self) -> Result<Vec<&str>, ApiMiscError> {
        // SAFETY:
        // Calling isValidListString is always safe.
        // Any bit pattern is expected by this function.
        if unsafe { !isValidListString(self) } {
            return Err(ApiMiscError::InvalidList);
        }
        if self.data.is_null() {
            return Ok(Vec::new());
        }

        if !self.data.is_aligned() {
            return Err(ApiMiscError::InvalidList)
        }
        let isize_len = self.length.try_into().map_err(|_error| ApiMiscError::InvalidList)?;

        if self.data.wrapping_offset(isize_len) > self.data {
            return Err(ApiMiscError::InvalidList);
        }
        // SAFETY:
        // The safety requirements listed in slice::from_raw_parts are checked by the above checks therefore calling the
        // function is safe to do.
        let slice = unsafe { slice::from_raw_parts(self.data, self.length) };
        slice
            .iter()
            .map(CString::as_str)
            .collect::<Result<_, _>>()
    }
}





impl<T> From<T> for CList_String
where
    T: Into<Box<[CString]>>,
{
    #[inline]
    fn from(value: T) -> Self {
        let boxed_list: Box<[_]> = value.into();
        if boxed_list.is_empty() {
            // SAFETY:
            // Calling emptyListString is always safe.
            return unsafe { emptyListString() };
        }
        let length = boxed_list.len();
        let leaked = Box::into_raw(boxed_list);
        let ptr = leaked.cast(); 
        //todo!("research rather this is valid");
        
        // SAFETY:
        // Calling createListString with a valid pointer it's corresponding length and drop function 
        // like we do here is safe according to the c-api.
        unsafe { createListString(ptr, length, Some(drop_list_string)) }
    }
}



impl Copy for CApiVersion {}

///
/// Two apis with same major and feature versions are always considered equal.
/// Patch version is purposefully ignored as it never contains any braking changes that would cause a runtime incompatibility.
///
impl PartialEq for CApiVersion {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.feature == other.feature
    }
}

impl CApiVersion {
    #[must_use]
    #[inline]
    #[expect(clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "the const implementation of the parser requires these operations")]
    pub const fn cargo() -> Self {
        let cargo = env!("CARGO_PKG_VERSION");
        let bytes = cargo.as_bytes();
        let mut index = 0;
        let mut major = 0;
        while index < bytes.len() {
            if bytes[index] == b'.' {
                major = to_u16(bytes, 0, index);
                break;
            }
            index += 1;
        }
        index += 1;
        let start = index;
        let mut feature = 0;
        while index < bytes.len() {
            if bytes[index] == b'.' {
                feature = to_u8(bytes, start, index);
                break;
            }

            index += 1;
        }
        index += 1;
        let patch = to_u8(bytes, index, bytes.len());
        Self {
            major,
            feature,
            patch,
        }
    }

    #[must_use]
    #[inline]
    pub const fn new(major: u16, feature: u8, patch: u8) -> Self {
        Self {
            major,
            feature,
            patch,
        }
    }

}

///
/// `ApiMiscError` is the public error type returned by all public functions performing fallible operations of this module.
/// 
#[derive(Error, Display, Debug)]
#[non_exhaustive]
pub enum ApiMiscError {
    InvalidList,
    InvalidString,
    Service(#[from]ServiceError),
    Utf8(#[from] Utf8Error),
}


#[expect(clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "the const implementation of the parser requires these operations")]
const fn to_u8(bytes: &[u8], start: usize, end: usize) -> u8 {
    let mut res = 0;
    let mut i = start;
    while i < end {
        res = 10u8 * res + (bytes[i] - b'0');
        i += 1;
    }
    res
}

#[expect(clippy::indexing_slicing, clippy::arithmetic_side_effects, 
    clippy::as_conversions, reason = "the const implementation of the parser requires these operations")]
#[expect(clippy::single_call_fn, reason = "the function is only used once in the parser but extracted into a function for readability")]
const fn to_u16(bytes: &[u8], start: usize, end: usize) -> u16 {
    let mut res: u16 = 0;
    let mut i = start;
    while i < end {
        res = 10u16 * res + (bytes[i] - b'0') as u16;
        i += 1;
    }
    res
}


#[doc(hidden)]
#[expect(clippy::single_call_fn, reason = "drop function for string lists is only used only once in From<Into<Box<[CString]>>> impl")]
unsafe extern "C" fn drop_list_string(list: *mut CString, length: usize) {
    // SAFETY:
    // The contract of CStringListDeallocFP function pointer is guaranteeing a valid ptr and length.
    // This function should only ever be passed to createListString where the CList_String implementation can ensure this guarantee.
    // Creating a slice from raw parts and Box from it is also safe as the CList_String abstraction in pair with the c-api
    // ensures that the only CList_String instance that call this function are those created from rust Box<[CString]>.
    let slice = unsafe { slice::from_raw_parts_mut(list, length) };
    // SAFETY: See safety block above.
    let owned = unsafe { Box::from_raw(slice) };
    drop(owned);
}

#[doc(hidden)]
#[expect(clippy::single_call_fn, reason = "drop function for strings is only used only once in From<Into<Box<str>>> impl")]
unsafe extern "C" fn drop_string(str: *const u8, length: usize) {
    // SAFETY: 
    // The contract of the CStringDeallocFP function pointer is guaranteeing a valid ptr and length.
    // This function should only ever be passed to createString where the CString implementation can ensure this guarantee.
    // Creating an utf8 slice and Box from it is also safe as the CString abstraction in pair with the c-api
    // ensure that the only CString instances that call this function are those created from rust Box<str>.
    let slice = unsafe { slice::from_raw_parts_mut(str.cast_mut(), length) };
    // SAFETY: See safety block above.
    let string = unsafe { str::from_utf8_unchecked_mut(slice) };
    // SAFETY: See safety block above.
    let owned = unsafe { Box::from_raw(string) };
    drop(owned);
}
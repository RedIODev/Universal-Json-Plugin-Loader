use std::str::Utf8Error;

use derive_more::Display;
use thiserror::Error;

use crate::{ErrorMapper, ServiceError, cbindings::{
    CApiVersion, CEventHandler, CList_String, CServiceError, CString, CUuid, asErrorString, createListString, createString, destroyListString, destroyString, emptyListString, fromErrorString, getLengthString, getViewString, isValidListString, isValidString
}};

#[cfg(feature = "safe")]
impl From<CUuid> for uuid::Uuid {
    fn from(value: CUuid) -> Self {
        Self::from_u64_pair(value.higher, value.lower)
    }
}

#[cfg(feature = "safe")]
impl From<uuid::Uuid> for CUuid {
    fn from(value: uuid::Uuid) -> Self {
        let (higher, lower) = value.as_u64_pair();
        Self { higher, lower }
    }
}

impl Drop for CString {
    fn drop(&mut self) {
        unsafe { destroyString(self) };
    }
}

impl CString {
    pub fn as_str(&self) -> Result<&str, ApiMiscError> {
        if unsafe { !isValidString(self) } {
            let service_error = unsafe { asErrorString(self) };
            if service_error.is_null() {
                return Err(ApiMiscError::InvalidString);
            }
            unsafe {(&*service_error).clone()}.to_rust()?;
        }
        let len = unsafe { getLengthString(self) };
        let ptr = unsafe { getViewString(self, 0, len) };
        Ok(std::str::from_utf8(unsafe {
            std::slice::from_raw_parts(ptr, len)
        })?)
    }
}

impl From<ServiceError> for CString {
    fn from(value: ServiceError) -> Self {
        unsafe {fromErrorString(value.into())}
    }
}

impl<'a> From<&'a CString> for Result<&'a str, ServiceError> {
    fn from(value: &'a CString) -> Self {
        value.as_str().map_err(|e| match e {
                ApiMiscError::Service(se) => se,
                e => Err(e).error(ServiceError::InvalidString).expect("unreachable!")
            }
        )
    }
}

impl From<CString> for Result<String, ServiceError> {
    fn from(value: CString) -> Self {
        Result::<& str,_>::from(&value).map(String::from)
    }
}

pub trait ToCString {
    fn to_c_string(self) -> CString;
}

impl<T: Into<Box<str>>> ToCString for Result<T, ServiceError> {
    fn to_c_string(self) -> CString {
        match self {
            Ok(str) => CString::from(str.into()),
            Err(e) => e.into()
        }
    }
}

impl<T: Into<Box<str>>> From<T> for CString {
    fn from(value: T) -> Self {
        let boxed = value.into();
        let leaked = unsafe { &mut *Box::into_raw(boxed) };
        let ptr = leaked.as_ptr();
        let length = leaked.len();
        unsafe { createString(ptr, length, Some(drop_string)) }
    }
}

unsafe extern "C" fn drop_string(str: *const u8, length: usize) {
    let slice = unsafe { std::slice::from_raw_parts_mut(str.cast_mut(), length) };
    let string = unsafe { std::str::from_utf8_unchecked_mut(slice) };
    let owned = unsafe { Box::from_raw(string) };
    drop(owned);
}

impl CEventHandler {
    #[must_use]
    pub fn new_error(error: CServiceError) -> Self {
        Self {
            function: None,
            handler_id: CUuid {
                higher: 0,
                lower: 0,
            },
            error,
        }
    }
}

impl Drop for CList_String {
    fn drop(&mut self) {
        unsafe { destroyListString(self) };
    }
}

impl CList_String {
    pub fn as_array(&self) -> Result<Vec<&str>, ApiMiscError> {
        if unsafe { !isValidListString(self) } {
            return Err(ApiMiscError::InvalidList);
        }
        if self.data.is_null() {
            return Ok(Vec::new());
        }
        let slice = unsafe { std::slice::from_raw_parts(self.data, self.length as usize) };
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
    fn from(value: T) -> Self {
        let boxed_list: Box<[_]> = value.into();
        if boxed_list.is_empty() {
            return unsafe { emptyListString() };
        }
        let Ok(length) = boxed_list.len().try_into() else {
            return unsafe { emptyListString() };
        };
        let leaked = unsafe { &mut *Box::into_raw(boxed_list) };
        let ptr = leaked.as_mut_ptr();
        
        unsafe { createListString(ptr, length, Some(drop_list_string)) }
    }
}

unsafe extern "C" fn drop_list_string(list: *mut CString, length: u32) {
    let slice = unsafe { std::slice::from_raw_parts_mut(list, length as usize) };
    let owned = unsafe { Box::from_raw(slice) };
    drop(owned);
}

impl Copy for CApiVersion {}

///
/// Two apis with same major and feature versions are always considered equal.
/// Patch version is purposefully ignored as it never contains any braking changes that would cause a runtime incompatibility.
///
impl PartialEq for CApiVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.feature == other.feature
    }
}

impl CApiVersion {
    #[must_use]
    pub const fn new(major: u16, feature: u8, patch: u8) -> Self {
        Self {
            major,
            feature,
            patch,
        }
    }

    #[must_use]
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
}

const fn to_u8(bytes: &[u8], start: usize, end: usize) -> u8 {
    let mut res = 0;
    let mut i = start;
    while i < end {
        res = 10u8 * res + (bytes[i] - b'0');
        i += 1;
    }
    res
}

const fn to_u16(bytes: &[u8], start: usize, end: usize) -> u16 {
    let mut res: u16 = 0;
    let mut i = start;
    while i < end {
        res = 10u16 * res + (bytes[i] - b'0') as u16;
        i += 1;
    }
    res
}

#[derive(Error, Display, Debug)]
pub enum ApiMiscError {
    InvalidList,
    InvalidString,
    Utf8(#[from] Utf8Error),
    Service(#[from]ServiceError)
}
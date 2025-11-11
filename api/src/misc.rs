use std::{str::Utf8Error};

use derive_more::Display;
use thiserror::Error;

use crate::{cbindings::{CApiVersion, CEndpointResponse, CEventHandler, CList_String, CServiceError, CString, CUuid, createListString, createString, destroyListString, destroyString, emptyListString, getLengthString, getViewString, isValidListString, isValidString}};

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
        Self { higher , lower }
    }
}

impl Drop for CString {
    fn drop(&mut self) {
        unsafe { destroyString(self) };
    }
}

#[derive(Error, Display, Debug)]
pub enum StringConventError {
    InvalidString,
    Utf8(#[from] Utf8Error),
}

impl CString {
    pub fn as_str(&self) -> Result<&str, StringConventError> {
        if unsafe { !isValidString(self) } {
            return Err(StringConventError::InvalidString);
        }
        let len = unsafe { getLengthString(self) };
        let ptr = unsafe { getViewString(self, 0, len) };
        Ok(std::str::from_utf8(unsafe {
            std::slice::from_raw_parts(ptr, len)
        })?)
    }
}

impl<T> From<T> for CString
where
    T: Into<Box<str>>,
{
    fn from(value: T) -> Self {
        let boxed_str: Box<str> = value.into();
        let leaked = unsafe { &mut *Box::into_raw(boxed_str) };
        let ptr = leaked.as_ptr();
        let length = leaked.len();
        unsafe { createString(ptr, length, Some(drop_string)) }
    }
}

unsafe extern "C" fn drop_string(str: *const u8, length: usize) {
    let slice = unsafe { std::slice::from_raw_parts_mut(str as *mut u8, length) };
    let string = unsafe { std::str::from_utf8_unchecked_mut(slice) };
    let owned = unsafe { Box::from_raw(string) };
    drop(owned);
}

impl CEventHandler {
    pub fn new_error(error: CServiceError) -> Self {
        Self {
            function: None,
            handler_id: CUuid { higher: 0, lower: 0},
            error,
        }
    }
}

impl CEndpointResponse {
    pub fn new_error(error: CServiceError) -> Self {
        Self {
            response: CString::from(""),
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
    pub fn as_array(&self) -> Result<Vec<&str>, StringListError> {
        if unsafe { !isValidListString(self) } {
            return Err(StringListError::InvalidList);
        }
        if self.data.is_null() {
            return Ok(Vec::new());
        }
        let slice = unsafe { std::slice::from_raw_parts(self.data, self.length as usize) };
        Ok(slice.iter().map(CString::as_str).collect::<Result<_, _>>()?)
    }
}


impl<T> From<T> for CList_String
where
    T: Into<Box<[CString]>>,
{
    fn from(value: T) -> Self {
        let boxed_list: Box<[_]> = value.into();
        if boxed_list.len() == 0 {
            return unsafe { emptyListString() }
        }
        let leaked = unsafe { &mut *Box::into_raw(boxed_list) };
        let ptr = leaked.as_mut_ptr();
        let length = leaked.len();
        unsafe { createListString(ptr, length as u32, Some(drop_list_string)) }
    }
}

unsafe extern "C" fn drop_list_string(list: *mut CString, length: u32) {
    let slice = unsafe { std::slice::from_raw_parts_mut(list, length as usize) };
    let owned = unsafe { Box::from_raw(slice) };
    drop(owned);
}

#[derive(Error, Display, Debug)]
pub enum StringListError {
    InvalidList,
    StringConventError(#[from]StringConventError)
}

impl Clone for CApiVersion {
    fn clone(&self) -> Self {
        *self
    }
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
    pub const fn new(major: u16, feature: u8, patch: u8) -> Self {
        Self { major, feature, patch }
    }

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
        index+=1;
        let patch = to_u8(bytes, index, bytes.len());
        Self { major, feature, patch }

    }
}

const fn to_u8(bytes: &[u8], start: usize, end: usize) -> u8 {
    let mut res = 0;
    let mut i = start;
    while i < end {
        res = 10u8 * res + (bytes[i] - b'0');
        i+=1;
    }
    res
}

const fn to_u16(bytes: &[u8], start: usize, end: usize) -> u16 {
    let mut res: u16 = 0;
    let mut i = start;
    while i < end {
        res = 10u16 * res + (bytes[i] - b'0') as u16;
        i+=1;
    }
    res
}


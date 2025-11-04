use std::str::Utf8Error;

use derive_more::Display;
use thiserror::Error;

use crate::{cbindings::{CString, CUuid, createString, destroyString, getLengthString, getViewString, isValidString}};


impl From<CUuid> for uuid::Uuid {
    fn from(value: CUuid) -> Self {
        Self::from_u64_pair(value.higher, value.lower)
    }
}

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
    let _ = unsafe { Box::from_raw(string) };
}
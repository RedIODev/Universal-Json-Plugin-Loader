use std::str::Utf8Error;

use derive_more::Display;
use thiserror::Error;

use crate::cbindings::{self, destroyString, getLengthString, getViewString, isValidString};

impl Drop for cbindings::String {
    fn drop(&mut self) {
        unsafe { destroyString(self) };
    }
}

impl cbindings::String {
    pub fn as_str(&self) -> Result<&str, StringConventError> {
        if unsafe {!isValidString(self)} {
        return Err(StringConventError::InvalidString);
        }
        let len = unsafe { getLengthString(self) };
        let ptr = unsafe { getViewString(self, 0, len)};
        Ok(std::str::from_utf8(unsafe { std::slice::from_raw_parts(ptr, len) })?)
    }
}

#[derive(Error, Display, Debug)]
pub enum StringConventError {
    InvalidString,
    Utf8(Utf8Error)
}
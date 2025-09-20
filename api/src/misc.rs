use std::{hash::Hash, str::Utf8Error};

use derive_more::Display;
use thiserror::Error;

use crate::cbindings::{createString, destroyString, getLengthString, getViewString, isValidString, CHandler, CHandlerFP, CString, CUuid, ServiceError};

impl Drop for CString {
    fn drop(&mut self) {
        unsafe { destroyString(self) };
    }
}

impl CString {
    pub fn as_str(&self) -> Result<&str, StringConventError> {
        if unsafe {!isValidString(self)} {
        return Err(StringConventError::InvalidString);
        }
        let len = unsafe { getLengthString(self) };
        let ptr = unsafe { getViewString(self, 0, len)};
        Ok(std::str::from_utf8(unsafe { std::slice::from_raw_parts(ptr, len) })?)
    }

    pub fn from_string(string: std::string::String) -> Self {
        let boxed_str = string.into_boxed_str();
        let leaked = unsafe { &mut  *Box::into_raw(boxed_str) };
        let ptr = leaked.as_ptr();
        let length = leaked.len();
        unsafe { createString(ptr, length, Some(drop_string)) }
    }
}

unsafe extern "C" fn drop_string(str: *const u8, length: usize) {
    let slice = unsafe { std::slice::from_raw_parts_mut(str as *mut u8, length)};
    let string = unsafe { std::str::from_utf8_unchecked_mut(slice)};
    let _ = unsafe { Box::from_raw(string) };
}



#[derive(Error, Display, Debug)]
pub enum StringConventError {
    InvalidString,
    Utf8(#[from]Utf8Error)
}

impl CUuid {
    pub fn as_u64_pair(&self) -> (u64, u64) {
        (self.first, self.second)
    }

    pub fn from_u64_pair(pair: (u64, u64)) -> Self {
        Self {first: pair.0, second: pair.1}
    }
}

impl Clone for CUuid {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for CUuid {

}

impl PartialEq for CUuid {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first && self.second == other.second
    }
}

impl Hash for CUuid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.first.hash(state);
        self.second.hash(state);
    }
}

impl Eq for CUuid {

}

pub trait OptionWrapped {
    type Unwrapped;
}

impl<T> OptionWrapped for Option<T> {
    type Unwrapped = T;
}

pub type HandlerFP = <CHandlerFP as OptionWrapped>::Unwrapped;

impl CHandler {
    pub fn new_error(error: ServiceError) -> Self {
        Self { function: None, handler_id: CUuid::from_u64_pair((0,0)), error }
    }
}

#[derive(Clone)]
pub struct Handler {
    pub function: HandlerFP,
    pub handler_id: CUuid,
}

impl TryFrom<CHandler> for Handler {
    type Error = ();

    fn try_from(value: CHandler) -> Result<Self, Self::Error> {
        let Some(function) = value.function else {
            return Err(());
        };
        Ok(Handler { function, handler_id: value.handler_id})
    }
}

impl From<Handler> for CHandler {
    fn from(value: Handler) -> Self {
        Self { function: Some(value.function), handler_id: value.handler_id, error: ServiceError::Success }
    }
}


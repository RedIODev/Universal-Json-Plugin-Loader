use std::{error::Error, hash::Hash, str::Utf8Error};

use derive_more::Display;
use thiserror::Error;

use crate::cbindings::{createString, destroyListString, destroyString, getLengthString, getViewString, isValidString, CEventHandler, CEventHandlerFP, CString, CUuid, List_String, ServiceError};

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
}

impl<T> From<T> for CString where T: Into<Box<str>> {
    fn from(value: T) -> Self {
        let boxed_str: Box<str> = value.into();
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

impl Drop for List_String {
    fn drop(&mut self) {
        unsafe { destroyListString(self) };
    }
}



#[derive(Error, Display, Debug)]
pub enum StringConventError {
    InvalidString,
    Utf8(#[from]Utf8Error)
}

impl CUuid {
    pub const fn as_u64_pair(&self) -> (u64, u64) {
        (self.first, self.second)
    }

    pub const fn from_u64_pair(pair: (u64, u64)) -> Self {
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

pub type EventHandlerFP = <CEventHandlerFP as OptionWrapped>::Unwrapped;

impl CEventHandler {
    pub fn new_error(error: ServiceError) -> Self {
        Self { function: None, handler_id: CUuid::from_u64_pair((0,0)), error }
    }
}

#[derive(Clone)]
pub struct EventHandler {
    pub function: EventHandlerFP,
    pub handler_id: CUuid,
}

impl TryFrom<CEventHandler> for EventHandler {
    type Error = ();

    fn try_from(value: CEventHandler) -> Result<Self, Self::Error> {
        let Some(function) = value.function else {
            return Err(());
        };
        Ok(EventHandler { function, handler_id: value.handler_id})
    }
}

impl From<EventHandler> for CEventHandler {
    fn from(value: EventHandler) -> Self {
        Self { function: Some(value.function), handler_id: value.handler_id, error: ServiceError::Success }
    }
}

impl Error for ServiceError {

}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<ServiceError> for Result<(), ServiceError> {
    fn from(value: ServiceError) -> Self {
        if value == ServiceError::Success {
            Ok(())
        } else { Err(value) }
    }
}
impl ServiceError {
    pub fn result(self) -> Result<(), ServiceError> {
        Result::from(self)
    }
}
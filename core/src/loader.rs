use std::{collections::HashMap, error::Error, ops::DerefMut, str::Utf8Error, sync::{Arc, LazyLock, Mutex, MutexGuard}};

use derive_more::Display;
use jsonschema::Validator;
use libloading::{Library, Symbol};

use anyhow::Result;

use finance_together_api::cbindings::{destroyString, getLengthString, getViewString, isValidString, u128_, ContextSupplier, Handler, HandlerRegisterService, String, Uuid};
use thiserror::Error;

pub struct Loader {
    libs: Vec<Library>,
}

impl Loader {

    pub fn new() -> Loader {
        Loader { libs: Vec::new() }
    }

    pub fn load_library(&mut self, filename: &str) -> Result<()> {
        let lib = unsafe { Library::new(filename)? };
        let main = unsafe { lib.get::<Symbol<unsafe extern "C" fn(HandlerRegisterService, Uuid)>>(b"pluginMain")?}; 
        let (lower, higher) = uuid::Uuid::new_v4().as_u64_pair();
        unsafe { main(Some(handlerRegister), Uuid { lower, higher }) };
        self.libs.push(lib);

        Ok(())
    }
}

static HANDLER: LazyLock<Mutex<HashMap<Box<str>, Event>>> = LazyLock::new(Mutex::default);

struct Event {
    handlers: Vec<StoredHandler>,
    validator: Validator
}

struct StoredHandler {
    handler: unsafe extern "C" fn(arg1: ContextSupplier, arg2: String, arg3: *mut String) -> bool,
    plugin_id: Uuid
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn handlerRegister(handler: Handler, plugin_id: Uuid, event_name: String) -> bool {
    let Some(handler) = handler else {
        return false;
    };
    let Ok(mut events) = HANDLER.lock() else {
        return false;
    };
    let Ok(event_name) = convert_string(event_name) else {
        return false;
    };
    let Some(event) = events.get_mut(&event_name) else {
        return false;
    };
    event.handlers.push(StoredHandler { handler, plugin_id });
    true
}



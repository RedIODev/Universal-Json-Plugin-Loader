use std::{collections::HashMap, sync::{LazyLock, Mutex}};

use derive_more::Display;
use jsonschema::Validator;
use libloading::{Library, Symbol};

use anyhow::Result;

use finance_together_api::cbindings::{CHandler, CUuid};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

use crate::runtime::{Event, StoredHandler};

pub struct Loader {
    libs: Vec<Library>,
    pub(crate)handler: LazyLock<Mutex<HashMap<Box<str>, Event>>>
}

impl Loader {

    pub const fn new() -> Loader {
        Loader { libs: Vec::new(), handler: LazyLock::new(|| Mutex::new(Loader::register_core_events()))}
    }

    pub unsafe fn load_library(&mut self, filename: &str) -> Result<()> {
        let lib = unsafe { Library::new(filename)? };
        let main = unsafe { lib.get::<Symbol<unsafe extern "C" fn(CUuid)->CHandler>>(b"pluginMain")?}; 
        let plugin_id = CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair());
        let init_handler = unsafe { main(plugin_id) };
        let Some(handler) = init_handler else {
            return Err(LoadError::NullInit.into());
        };
        let Ok(mut events) = self.handler.lock() else {
            return Err(LoadError::Internal.into());
        };
        
        let init = events.get_mut("core:init").expect("core event missing!");
        init.handlers.push(StoredHandler { handler, plugin_id });
        self.libs.push(lib);

        Ok(())
    }

    fn register_core_events() -> HashMap<Box<str>, Event> {
        let mut hashmap = HashMap::new();
        hashmap.insert("core:init".into(), 
            Event::new(
                Loader::schema_from_file(include_str!("../event/init-args.json")),
                Loader::schema_from_file(include_str!("../event/init-result.json"))
            )
        );
        hashmap
    }

    fn schema_from_file(file:&str) -> Validator {
        jsonschema::validator_for(&json!(file)).expect("invalid schema!")

    }
}

#[derive(Error, Debug, Display)]
enum LoadError {
    NullInit,
    Internal
}




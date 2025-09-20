use std::collections::HashMap;

use derive_more::Display;
use libloading::{Library, Symbol};

use anyhow::Result;

use finance_together_api::{cbindings::{CHandlerFP, CUuid}, Handler};
use thiserror::Error;
use uuid::Uuid;

use crate::runtime::{Event, Runtime, StoredHandler};

pub struct Loader {
    libs: Vec<Library>,
    pub(crate)events: HashMap<Box<str>, Event>,
    runtime: Runtime
}

impl Loader {

    pub fn new(runtime: Runtime) -> Loader {
        Loader { libs: Vec::new(), events: runtime.register_core_events(), runtime}
    }

    pub unsafe fn load_library(&mut self, filename: &str) -> Result<()> {
        let lib = unsafe { Library::new(filename)? };
        let main = unsafe { lib.get::<Symbol<unsafe extern "C" fn(CUuid)->CHandlerFP>>(b"pluginMain")?}; 
        let plugin_id = CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair());
        let init_handler = unsafe { main(plugin_id) };
        let Some(init_handler) = init_handler else {
            return Err(LoadError::NullInit.into());
        };

        let init = self.events.get_mut("core:init").expect("core events missing!");
        init.handlers.insert(StoredHandler::new(Handler { function: init_handler, handler_id: CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair())}, plugin_id));
        self.libs.push(lib);

        Ok(())
    }
}

#[derive(Error, Debug, Display)]
enum LoadError {
    NullInit
}




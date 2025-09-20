use derive_more::Display;
use libloading::{Library, Symbol};

use anyhow::Result;

use finance_together_api::{cbindings::{CHandlerFP, CUuid}, Handler};
use thiserror::Error;
use uuid::Uuid;

use crate::{runtime:: StoredHandler, GGL};

pub struct Loader {
    libs: Vec<Library>,
    
}

impl Loader {

    pub fn new() -> Loader {
        Loader { libs: Vec::new()}
    }

    pub unsafe fn load_library(filename: &str) -> Result<()> {
        let lib = unsafe { Library::new(filename)? };
        let main = unsafe { lib.get::<Symbol<unsafe extern "C" fn(CUuid)->CHandlerFP>>(b"pluginMain")?}; 
        let plugin_id = CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair());
        let init_handler = unsafe { main(plugin_id) };
        let Some(init_handler) = init_handler else {
            return Err(LoadError::NullInit.into());
        };
        { // Mutex start
            let Ok(mut gov) = GGL.lock() else {
                return Err(LoadError::Internal.into());
            };
            let init = gov.events_mut().get_mut("core:init").expect("core events missing!");
            init.handlers.insert(StoredHandler::new(Handler { function: init_handler, handler_id: CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair())}, plugin_id));
            gov.libs_mut().push(lib);
        } // Mutex end
        Ok(())
    }

    pub fn libs_mut(&mut self) -> &mut Vec<Library> {
        &mut self.libs
    }
}

#[derive(Error, Debug, Display)]
enum LoadError {
    NullInit,
    Internal
}




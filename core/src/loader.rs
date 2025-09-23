use std::collections::HashMap;

use derive_more::Display;
use libloading::{Library, Symbol};

use anyhow::Result;

use finance_together_api::{cbindings::{CUuid, PluginInfo}, EventHandler};
use thiserror::Error;
use uuid::Uuid;

use crate::{runtime::event::StoredEventHandler, GGL};

pub struct Loader {
    plugins: HashMap<CUuid, Plugin>,
    
}

#[derive(Debug)]
pub struct Plugin {
    _lib: Library,
    pub name: Box<str>,
    pub version: Box<str>
}

impl Loader {

    pub fn new() -> Loader {
        Loader { plugins: HashMap::new()}
    }

    pub unsafe fn load_library(filename: &str) -> Result<()> {
        let lib = unsafe { Library::new(filename)? };
        let main = unsafe { lib.get::<Symbol<unsafe extern "C" fn(CUuid)->PluginInfo>>(b"pluginMain")?}; 
        let plugin_id = CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair());
        let plugin_info = unsafe { main(plugin_id) };
        let plugin = Plugin { _lib: lib, name: plugin_info.name.as_str()?.into(), version: plugin_info.version.as_str()?.into() };
        let plugin_name = plugin.name.clone();
        let plugin_version = plugin.version.clone();
        let Some(init_handler) = plugin_info.init_handler else {
            return Err(LoadError::NullInit.into());
        };

        { // Mutex start
            let Ok(mut gov) = GGL.write() else {
                return Err(LoadError::Internal.into());
            };
           
            if gov.plugins().values().find(|p|p.name == plugin.name).is_some() {
                return Err(LoadError::DuplicateName.into());
            }
            let init = gov.events_mut().get_mut("core:init").expect("core events missing!");
            init.handlers.insert(StoredEventHandler::new(EventHandler { function: init_handler, handler_id: CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair())}, plugin_id));
            gov.plugins_mut().insert(plugin_id, plugin);
        } // Mutex end
        println!("Loaded Plugin \"{}\" version: {}", plugin_name, plugin_version);
        Ok(())
    }

    pub fn plugins_mut(&mut self) -> &mut HashMap<CUuid, Plugin> {
        &mut self.plugins
    }

    pub fn plugins(&self) -> & HashMap<CUuid, Plugin> {
        & self.plugins
    }
}

#[derive(Error, Debug, Display)]
enum LoadError {
    NullInit,
    Internal,
    DuplicateName
}




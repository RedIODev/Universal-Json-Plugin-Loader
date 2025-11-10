use std::sync::Arc;

use arc_swap::ArcSwap;
use derive_more::Display;
use finance_together_api::{API_VERSION, EventHandler, ServiceError, cbindings::{CPluginInfo, CUuid}};
use libloading::{Library, Symbol};

use anyhow::Result;

use thiserror::Error;
use uuid::Uuid;

use crate::{
    governor::get_gov,
    runtime::event::StoredEventHandler,
    util::{ArcMapExt, LockedMap, TrueOrErr},
};

pub type Plugins = LockedMap<Uuid, Plugin>;

pub struct Loader {
    plugins: LockedMap<Uuid, Plugin>,
}

#[derive(Clone)]
pub struct Plugin {
    _lib: Arc<Library>,
    pub name: Arc<str>,
    pub version: Box<str>,
    pub dependencies: Box<[Box<str>]>,
}

impl Loader {
    pub fn new() -> Loader {
        Loader {
            plugins: ArcSwap::default(),
        }
    }

    pub unsafe fn load_library(filename: &str) -> Result<()> {
        let lib = unsafe { Library::new(filename)? };
        let main =
            unsafe { lib.get::<Symbol<unsafe extern "C" fn(CUuid) -> CPluginInfo>>(b"plugin_main")? };
        let plugin_id = Uuid::new_v4();
        let plugin_info = unsafe { main(plugin_id.into()) };
        if plugin_info.api_version != API_VERSION {
            return Err(LoadError::ApiVersion.into())
        }
        let dependencies = plugin_info
            .dependencies
            .as_array()?
            .into_iter()
            .map(Box::from)
            .collect();
        let plugin = Plugin {
            _lib: Arc::new(lib),
            name: plugin_info.name.as_str()?.into(),
            version: plugin_info.version.as_str()?.into(),
            dependencies,
        };
        let plugin_name = plugin.name.clone();
        let plugin_version = plugin.version.clone();
        let Some(init_handler) = plugin_info.init_handler else {
            return Err(LoadError::NullInit.into());
        };

        {
            // Mutex start
            let Ok(gov) = get_gov() else {
                return Err(LoadError::Internal.into());
            };

            if gov
                .loader()
                .plugins()
                .load()
                .values()
                .find(|p| p.name == plugin.name)
                .is_some()
            {
                return Err(LoadError::DuplicateName.into());
            }
            let handler = StoredEventHandler::new(
                EventHandler::new_unsafe(init_handler, Uuid::new_v4()),
                plugin_id,
            );
            gov.events()
                .rcu_alter("core:init", |event| {
                    event
                        .handlers
                        .insert(handler.clone())
                        .or_error(ServiceError::CoreInternalError)
                })
                .expect("core events missing!");
            gov.loader()
                .plugins()
                .rcu(|map| map.update(plugin_id, plugin.clone()));
        } // Mutex end
        println!(
            "Loaded Plugin \"{}\" version: {}",
            plugin_name, plugin_version
        );
        Ok(())
    }

    pub fn plugins(&self) -> &Plugins {
        &self.plugins
    }

    pub fn load_libraries() -> Result<()> {
        unsafe { Loader::load_library("libexample.so")? };
        Ok(())
    }
}

#[derive(Error, Debug, Display)]
enum LoadError {
    NullInit,
    Internal,
    DuplicateName,
    ApiVersion
}

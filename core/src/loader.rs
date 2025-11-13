use std::sync::Arc;

use derive_more::Display;
use finance_together_api::{API_VERSION, EventHandler, ServiceError, cbindings::{CPluginInfo, CUuid}, misc::ApiMiscError};
use libloading::{Library, Symbol};


use thiserror::Error;
use uuid::Uuid;

use crate::{
    governor::get_gov,
    runtime::event::StoredEventHandler,
    util::{ArcMapExt, LockedMap, TrueOrErr},
};

pub type Plugins = LockedMap<Uuid, Plugin>;

#[derive(Default)]
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

    pub unsafe fn load_library(filename: &str) -> Result<(), LoaderError> {
        let lib = unsafe { Library::new(filename)? };
        let main =
            unsafe { lib.get::<Symbol<unsafe extern "C" fn(CUuid) -> CPluginInfo>>(b"plugin_main")? };
        let plugin_id = Uuid::new_v4();
        let plugin_info = unsafe { main(plugin_id.into()) }.to_rust()?;
        if plugin_info.api_version() != API_VERSION {
            return Err(LoaderError::ApiVersion)
        }
        let dependencies = plugin_info
            .dependencies()?
            .into_iter()
            .map(Box::from)
            .collect();
        let plugin = Plugin {
            _lib: Arc::new(lib),
            name: plugin_info.name()?.into(),
            version: plugin_info.version()?.into(),
            dependencies,
        };

        if plugin.name.contains(':') {
            return Err(LoaderError::InvalidName);
        }
        if &*plugin.name == "core" {
            return Err(LoaderError::InvalidName);
        }
        let init_handler = plugin_info.handler();

        {
            // Mutex start
            let Ok(gov) = get_gov() else {
                return Err(LoaderError::Internal);
            };

            if gov
                .loader()
                .plugins()
                .load()
                .values()
                .any(|p| p.name == plugin.name)
            {
                return Err(LoaderError::DuplicateName);
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

        #[cfg(debug_assertions)]
        println!(
            "Loaded Plugin \"{}\" version: {}",
            plugin.name, plugin.version
        );
        Ok(())
    }

    pub fn plugins(&self) -> &Plugins {
        &self.plugins
    }

    pub fn load_libraries() -> Result<(), LoaderError> {
        unsafe { Loader::load_library("libexample.so")? };
        Ok(())
    }
}

#[derive(Error, Debug, Display)]
pub enum LoaderError {
    Internal,
    DuplicateName,
    ApiVersion,
    InvalidName,
    LibError(#[from]libloading::Error),
    ServiceError(#[from]ServiceError),
    ApiMiscError(#[from]ApiMiscError)
}

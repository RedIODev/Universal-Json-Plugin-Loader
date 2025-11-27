use std::{fs};
use alloc::sync::Arc;
use derive_more::Display;
use plugin_loader_api::{API_VERSION, EventHandler, ServiceError, cbindings::{CPluginInfo, CUuid}, misc::ApiMiscError};
use libloading::{Library, Symbol};

use std::io;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    config::ConfigError, governor::{GovernorError, get_gov}, runtime::event::StoredEventHandler, util::{ArcMapExt as _, LockedMap, TrueOrErr as _}
};

pub type Plugins = LockedMap<Uuid, Plugin>;

#[derive(Default)]
pub struct Loader {
    plugins: LockedMap<Uuid, Plugin>,
}

#[derive(Clone)]
pub struct Plugin {
    _lib: Arc<Library>,
    dependencies: Box<[Box<str>]>,
    name: Arc<str>,
    version: Box<str>,
}

impl Plugin {
    pub const fn dependencies(&self) -> &[Box<str>] {
        &self.dependencies
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn version(&self) -> &str {
        &self.version
    }
}

impl Loader {
    #[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
    pub fn load_libraries() -> Result<(), LoaderError> {
        let plugin_folder = get_gov()?.config().config_dir()?.join("plugins");
        fs::create_dir_all(&plugin_folder)?;
        let plugins = plugin_folder.read_dir()?
                .filter_map(Result::ok)
                .map(|dir| dir.path())
                .filter(|entry| entry.is_file())
                .filter_map(|file| file.into_os_string().into_string().ok());
        for plugin in plugins {
            // SAFETY:
            // load_library is inherently unsafe as it calls foreign code.
            // The only safety we have is that we trust the path of the plugin_folder.
            unsafe { Self::load_library(&plugin)? }
        }   
        Ok(())
    }

    #[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
    unsafe fn load_library(filename: &str) -> Result<(), LoaderError> {
        // SAFETY:
        // Can't do anything to make loading a library safer.
        let lib = unsafe { Library::new(filename)? };
        let main =
        // SAFETY:
        // Finding the symbol plugin_main implies that the library is implementing
        // a plugin and therefore should provide the correct api according to the c-api.
        unsafe { lib.get::<Symbol<unsafe extern "C" fn(CUuid) -> CPluginInfo>>(b"plugin_main")? };
        let plugin_id = Uuid::new_v4();
        // SAFETY:
        // Calling plugin_main with the given arguments and return value is defined
        // by the c-api and therefore considered to be expected to work.
        let plugin_info = unsafe { main(plugin_id.into()) }.to_rust()?;
        if plugin_info.api_version() != API_VERSION {
            return Err(LoaderError::ApiVersion)
        }
        let dependencies = plugin_info
            .dependencies()?
            .into_iter()
            .map(Box::from)
            .collect();
        let new_plugin = Plugin {
            _lib: Arc::new(lib),
            name: plugin_info.name()?.into(),
            version: plugin_info.version()?.into(),
            dependencies,
        };

        if new_plugin.name.contains(':') {
            return Err(LoaderError::InvalidName);
        }
        if &*new_plugin.name == "core" {
            return Err(LoaderError::InvalidName);
        }
        let init_handler = plugin_info.handler();

            if get_gov()?
                .loader()
                .plugins()
                .load()
                .values()
                .any(|plugin| plugin.name == new_plugin.name)
            {
                return Err(LoaderError::DuplicateName);
            }
            let handler = StoredEventHandler::new(
                EventHandler::new_unsafe(init_handler, Uuid::new_v4()),
                plugin_id,
            );
            get_gov()?.events()
                .rcu_alter("core:init", |event| {
                    event
                        .handlers_mut()
                        .insert(handler.clone())
                        .or_error(ServiceError::CoreInternalError)
                }).map_err(|_error| LoaderError::CoreEventsMissing)?;
            get_gov()?.loader()
                .plugins()
                .rcu(|map| map.update(plugin_id, new_plugin.clone()));


        #[expect(clippy::print_stderr, reason = "debug_assertions")]
        #[cfg(debug_assertions)]
        {
            eprintln!(
                "Loaded Plugin \"{}\" version: {}",
                new_plugin.name, new_plugin.version
            );
        }
        Ok(())
    }

    pub const fn plugins(&self) -> &Plugins {
        &self.plugins
    }
}

#[derive(Error, Debug, Display)]
pub enum LoaderError {
    ApiMiscError(#[from]ApiMiscError),
    ApiVersion,
    ConfigError(#[from]ConfigError),
    CoreEventsMissing,
    DuplicateName,
    Governor(#[from]GovernorError),
    IO(#[from]io::Error),
    InvalidName,
    LibError(#[from]libloading::Error),
    ServiceError(#[from]ServiceError),
}

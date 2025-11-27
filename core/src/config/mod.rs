
pub mod cli;

use alloc::{borrow::Cow, sync::Arc};
use core::str::FromStr as _;
use core::ops::Deref as _;
use std::{
    collections::HashMap, env, fs, io, path::Path
};

use atomic_once_cell::AtomicOnceCell;
use clap::Args;
use convert_case::{Boundary, Case};
use derive_more::Display;
use plugin_loader_api::{
    ApplicationContext, ErrorMapper as _, ServiceError,
    pointer_traits::{RequestHandlerFunc, trait_fn},
};
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use toml::Table;
use toml::de::Error as TomlError;

use crate::{
    config::cli::{CliError, PluginOption},
    governor::{GovernorError, get_gov},
    util::{LockedMap, MapExt as _},
};

pub type ConfigMap = LockedMap<Box<str>, Table>;

#[derive(Default)]
pub struct Config {
    configs: ConfigMap,
    root_dir_name: AtomicOnceCell<Box<Path>>,
}

#[derive(Debug, Clone, Args)]
struct PluginArgs {
    plugin_args: Vec<String>,
}


#[derive(Debug, Display, Error)]
pub enum ConfigError {
    CliError(#[from] CliError),
    ConfigRootAlreadySet,
    DeserializeError(#[from] TomlError),
    GovernorError(#[from] GovernorError),
    IOError(#[from] io::Error),
    InvalidFileName,
    NoConfigDir,
}

impl Config {
    pub fn config_dir(&self) -> Result<&Path, ConfigError> {
        self.root_dir_name.get()
                .ok_or(ConfigError::NoConfigDir)
                .map(Box::deref)
    }
    
    fn env_prefix(&self) -> Result<Box<str>, ConfigError> {
        let dir = self.config_dir()?.file_name().ok_or(ConfigError::NoConfigDir)?;
        let converted = convert_case::Casing::to_case(&dir.to_string_lossy(), Case::Constant);
        Ok(convert_case::split(&converted, &[Boundary::Underscore]).iter().filter_map(|word| word.chars().nth(0)).collect())
    }

    pub fn init() -> Result<(), ConfigError> {
        let file_configs = Self::parse_files()?;
        let env_overrides = PluginOption::join_table(&Self::parse_env()?);
        let cli_overrides = PluginOption::join_table(&**get_gov()?.cli().plugins().load());

        let env_file_configs = file_configs.join_merge(env_overrides, |_, file, env| {
            file.join_merge(env, |_, _, env_val| env_val)
        });
        let cli_env_file_configs = env_file_configs.join_merge(cli_overrides, |_, org, cli| {
            org.join_merge(cli, |_, _, cli_val| cli_val)
        });

        get_gov()?
            .config()
            .configs
            .store(Arc::new(im::HashMap::from(cli_env_file_configs)));
        Ok(())
    }
    
    #[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
    fn parse_env() -> Result<Vec<PluginOption>, ConfigError> {
        let prefix = get_gov()?.config().env_prefix()?;
        Ok(env::vars()
            .filter_map(|(key, value)| key.strip_prefix(&*prefix).map(|stripped_key| format!("{stripped_key}={value}")))
            .map(|arg| PluginOption::from_str(&arg))
            .collect::<Result<Vec<_>, cli::CliError>>()?)
    }

    #[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
    fn parse_files() -> Result<HashMap<Box<str>, Table>, ConfigError> {
        let config_dir = get_gov()?.config().config_dir()?.join("config");
        fs::create_dir_all(&config_dir)?;
        config_dir.read_dir()?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "toml")
            })
            .map(|config_file| {
                Ok((
                    config_file
                        .file_name()
                        .to_str()
                        .ok_or(ConfigError::InvalidFileName)?
                        .into(),
                    Self::read_config(&config_file.path())?,
                ))
            })
            .collect::<Result<HashMap<Box<str>, _>, _>>()
    }

    fn read_config(path: &Path) -> Result<Table, ConfigError> {
        let config = fs::read_to_string(path)?;
        Ok(toml::from_str(&config)?)
    }
    pub fn set_config_dir<P: AsRef<Path>>(dir_name: P) -> Result<(), ConfigError> {
        let mut path = dir_name.as_ref().to_owned();
        if !path.is_absolute() {
            path = dirs::config_dir()
            .ok_or(ConfigError::NoConfigDir)?.join(path);
        }

        get_gov()?.config().root_dir_name.set(path.into()).map_err(|_error| ConfigError::ConfigRootAlreadySet)?;
        Ok(())
    }

}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Action {
    Load,
    Reload,
    Save,
}

#[derive(Deserialize)]
struct ConfigArgs {
    action: Action,
    key: Option<String>,
    value: Option<toml::Value>,
}

#[trait_fn(RequestHandlerFunc for ConfigRequestHandler)]
pub fn handle<'args, F: Fn() -> Result<ApplicationContext, ServiceError>, S: Into<Cow<'args, str>>, T: AsRef<str>>(
    _: F,
    plugin_name: T,
    args: S,
) -> Result<String, ServiceError> {
    let config_args = serde_json::from_str::<ConfigArgs>(&args.into()).error(ServiceError::InvalidJson)?;
    match config_args {
        ConfigArgs { action: Action::Load ,key, .. } => load_config(plugin_name.as_ref(), key),
        ConfigArgs { action: Action::Save, key: Some(key), value: Some(value) } => save_config(plugin_name.as_ref(), key, value),
        ConfigArgs { action: Action::Reload, .. } => reload_config(),
        _ => Err(ServiceError::InvalidApi)
    }
}

#[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
fn load_config(plugin_name: &str, key_opt: Option<String>) -> Result<String, ServiceError> {
    let gov = get_gov().error(ServiceError::CoreInternalError)?;
    let config = gov.config().configs.load();
    let conf = config
            .get(plugin_name)
            .error(ServiceError::NotFound)?;
    let Some(key) = key_opt else {
        return serde_json::to_string(conf).error(ServiceError::CoreInternalError);
    };
    let entry = conf.get(key.as_str()).error(ServiceError::NotFound)?;
    serde_json::to_string(entry).error(ServiceError::CoreInternalError)
}

#[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
fn save_config(plugin_name: &str, key: String, value: toml::Value) -> Result<String, ServiceError> {
    let filepath = get_gov()
            .error(ServiceError::CoreInternalError)?
            .config()
            .config_dir().error(ServiceError::CoreInternalError)?.join(plugin_name)
            .with_extension(".toml");
    let mut file_conf = Config::read_config(&filepath).error(ServiceError::CoreInternalError)?;
    file_conf.insert(key, value);
    fs::write(&filepath, 
        toml::to_string(&file_conf)
                    .error(ServiceError::CoreInternalError)?)
            .error(ServiceError::CoreInternalError)?;
    Ok(json!({}).to_string())
}

#[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
fn reload_config() -> Result<String, ServiceError> {
    Config::init().error(ServiceError::CoreInternalError)?;
    Ok(json!({}).to_string())
}

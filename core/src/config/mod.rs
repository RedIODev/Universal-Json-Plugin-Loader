pub mod cli;

use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    path::Path,
    str::FromStr,
    sync::Arc,
};

use atomic_once_cell::AtomicOnceCell;
use clap::Args;
use convert_case::{Boundary, Case};
use derive_more::Display;
use finance_together_api::{
    ApplicationContext, ErrorMapper, ServiceError,
    pointer_traits::{RequestHandlerFunc, trait_fn},
};
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use toml::Table;

use crate::{
    config::cli::{CliError, PluginOption},
    governor::{GovernorError, get_gov},
    util::{LockedMap, MapExt},
};

pub type ConfigMap = LockedMap<Box<str>, Table>;

#[derive(Default)]
pub struct Config {
    root_dir_name: AtomicOnceCell<Box<Path>>,
    configs: ConfigMap,
}

#[derive(Debug, Clone, Args)]
struct PluginArgs {
    plugin_args: Vec<String>,
}


impl Config {
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

    pub fn set_config_dir(dir_name: impl AsRef<Path>) -> Result<(), ConfigError> {
        let mut path = dir_name.as_ref().to_owned();
        if !path.is_absolute() {
            path = dirs::config_dir()
            .ok_or(ConfigError::NoConfigDir)?.join(path);
        }

        get_gov()?.config().root_dir_name.set(path.into()).map_err(|_| ConfigError::ConfigRootAlreadySet)?;
        Ok(())
    }

    pub fn config_dir(&self) -> &Path {
        self.root_dir_name.get().expect("No config dir specified!")
    }

    fn env_prefix(&self) -> Result<Box<str>, ConfigError> {
        let dir = self.config_dir().file_name().ok_or(ConfigError::NoConfigDir)?;
        let converted = convert_case::Casing::to_case(&dir.to_string_lossy(), Case::Constant);
        Ok(convert_case::split(&converted, &[Boundary::Underscore]).iter().filter_map(|word| word.chars().nth(0)).collect())
    }

    fn read_config(path: &Path) -> Result<Table, ConfigError> {
        let config = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&config)?)
    }

    fn parse_env() -> Result<Vec<PluginOption>, ConfigError> {
        let prefix = get_gov()?.config().env_prefix()?;
        Ok(std::env::vars()
            .filter_map(|(key, value)| key.strip_prefix(&*prefix).map(|key| format!("{key}={value}")))
            .map(|arg| PluginOption::from_str(&arg))
            .collect::<Result<Vec<_>, cli::CliError>>()?)
    }

    fn parse_files() -> Result<HashMap<Box<str>, Table>, ConfigError> {
        let config_dir = get_gov()?.config().config_dir().join("config");
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
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Action {
    Load,
    Save,
    Reload,
}

#[derive(Deserialize)]
struct ConfigArgs {
    action: Action,
    key: Option<String>,
    value: Option<toml::Value>,
}

#[trait_fn(RequestHandlerFunc for ConfigRequestHandler)]
pub fn handle<'a, F: Fn() -> ApplicationContext, S: Into<Cow<'a, str>>, T: AsRef<str>>(
    _: F,
    plugin_name: T,
    args: S,
) -> Result<String, ServiceError> {
    let args = serde_json::from_str::<ConfigArgs>(&args.into()).error(ServiceError::InvalidJson)?;
    match args {
        ConfigArgs { action: Action::Load ,key, .. } => load_config(plugin_name.as_ref(), key),
        ConfigArgs { action: Action::Save, key: Some(k), value: Some(v) } => save_config(plugin_name.as_ref(), k, v),
        ConfigArgs { action: Action::Reload, .. } => reload_config(),
        _ => Err(ServiceError::InvalidApi)
    }
}

fn load_config(plugin_name: &str, key: Option<String>) -> Result<String, ServiceError> {
    let gov = get_gov().error(ServiceError::CoreInternalError)?;
    let config = gov.config().configs.load();
    let conf = config
            .get(plugin_name)
            .error(ServiceError::NotFound)?;
    let Some(key) = key else {
        return serde_json::to_string(conf).error(ServiceError::CoreInternalError);
    };
    let entry = conf.get(key.as_str()).error(ServiceError::NotFound)?;
    serde_json::to_string(entry).error(ServiceError::CoreInternalError)
}

fn save_config(plugin_name: &str, key: String, value: toml::Value) -> Result<String, ServiceError> {
    let filepath = get_gov()
            .error(ServiceError::CoreInternalError)?
            .config()
            .config_dir().join(plugin_name)
            .with_extension(".toml");
    let mut file_conf = Config::read_config(&filepath).error(ServiceError::CoreInternalError)?;
    file_conf.insert(key, value);
    std::fs::write(&filepath, 
        toml::to_string(&file_conf)
                    .error(ServiceError::CoreInternalError)?)
            .error(ServiceError::CoreInternalError)?;
    Ok(json!({}).to_string())
}

fn reload_config() -> Result<String, ServiceError> {
    Config::init().error(ServiceError::CoreInternalError)?;
    Ok(json!({}).to_string())
}

#[derive(Debug, Display, Error)]
pub enum ConfigError {
    NoConfigDir,
    ConfigRootAlreadySet,
    InvalidFileName,
    CliError(#[from] CliError),
    GovernorError(#[from] GovernorError),
    IOError(#[from] std::io::Error),
    DeserializeError(#[from] toml::de::Error),
}

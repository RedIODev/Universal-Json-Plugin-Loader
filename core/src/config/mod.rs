pub mod cli;

use std::{collections::HashMap, fs, path::{Path, PathBuf}, str::FromStr, sync::Arc};

use anyhow::Result;
use arc_swap::ArcSwap;
use clap::Args;
use derive_more::Display;
use dirs::config_dir;
use thiserror::Error;
use toml::Table;


use crate::{config::cli::PluginOption, governor::get_gov, util::{LockedMap, MapExt}};

pub type ConfigMap = LockedMap<Box<str>, Table>;

pub struct Config {
    user_dir: PathBuf,
    configs: ConfigMap
}


#[derive(Debug, Clone, Args)]
struct PluginArgs {
    plugin_args: Vec<String>
}

impl Config {
    pub fn new() -> Self {
        let user_dir = config_dir()
            .ok_or(NoConfigDirError)
            .expect("No config dir found!")
            .join("finance-together");
        Self { user_dir, configs: ArcSwap::default() }
    }

    pub fn init() -> Result<()> {
        let file_configs = Self::parse_files()?;
        let env_overrides = PluginOption::join_table(&Self::parse_env()?);
        let cli_overrides = PluginOption::join_table(&**get_gov()?.cli().plugins().load());

        let env_file_configs = file_configs.join_merge(env_overrides, 
                |_, file, env| file.join_merge(env, |_,_, env_val| env_val));
        let cli_env_file_configs = env_file_configs.join_merge(cli_overrides, 
                |_, org, cli| org.join_merge(cli, |_, _, cli_val| cli_val));
        
        get_gov()?.config().configs.store(Arc::new(im::HashMap::from(cli_env_file_configs)));
        Ok(())
    }

    fn read_config(path: &Path) -> Result<Table> {
        let config = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&config)?)
    }

    fn parse_env() -> Result<Vec<PluginOption>, cli::ParseError> {
        std::env::vars()
                .filter_map(|(key, value)| key.strip_prefix("FT_").map(|key| format!("{key}={value}")))
                .map(|arg| PluginOption::from_str(&arg))
                .collect::<Result<Vec<_>, cli::ParseError>>()
    }

    fn parse_files() -> Result<HashMap<Box<str>, Table>> {
        let config_dir = get_gov()?.config().user_dir.join("config");
        fs::create_dir_all(config_dir.clone())?;
        fs::read_dir(config_dir)?
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_file())
                .filter(|entry| entry.path().extension().map(|extension| extension == "toml").unwrap_or(false))
                .map(|config_file| Ok((config_file.file_name().to_str().ok_or(InvalidFileName)?.into(), Self::read_config(&config_file.path())?)))
                .collect::<Result<HashMap<Box<str>,_>>>()
    }
}

#[derive(Debug, Display, Error)]
pub struct NoConfigDirError;

#[derive(Debug, Display, Error)]
pub struct InvalidFileName;
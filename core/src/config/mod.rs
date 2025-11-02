pub mod cli;

use std::{collections::HashMap, fs, path::{Path, PathBuf}};

use anyhow::Result;
use arc_swap::ArcSwap;
use clap::{Args, Parser};
use convert_case::{Case, Casing};
use derive_more::Display;
use dirs::config_dir;
use figment::{providers::Env, Figment};
use thiserror::Error;
use toml::Table;


use crate::{config::cli::{Cli, CliPluginOption}, governor::get_gov, util::LockedMap};

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
        let config_dir = get_gov()?.config().user_dir.join("config");
        fs::create_dir_all(config_dir.clone())?;
        let config_files = fs::read_dir(config_dir)?
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_file())
                .filter(|entry| entry.path().extension().map(|extension| extension == "toml").unwrap_or(false))
                .map(|config_file| Ok((config_file.file_name().to_str().ok_or(InvalidFileName)?.into(), Self::read_config(&config_file.path())?)))
                .collect::<Result<HashMap<Box<str>,_>>>()?;
                //.map() //create config for each file and return im::Hashmap from iter and rcu into gov
        let override_configs = CliPluginOption::join_table(&**get_gov()?.cli().plugins().load());
        { //Mutex start
            let gov = get_gov()?;
            
        } //Mutex end
        Ok(())
    }

    fn read_config(path: &Path) -> Result<Table> {
        let config = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&config)?)
    }

    fn base_figment(filename: &str) -> Figment {
        Figment::new()
            .merge(Env::prefixed(&filename.to_case(Case::Constant)))
      
    }

    //add overrides to builder with plugin prefix removed and prefix mapped to category

    // fn base_builder(filename: &str) -> ConfigBuilder {
    //     let env = config::Environment::with_prefix(&filename.to_uppercase());
    //     let override_conf = config::Config::builder().add_source(env).build().unwrap();

    //     config::Config::builder()
    //         .

    // } 
}

#[derive(Debug, Display, Error)]
pub struct NoConfigDirError;

#[derive(Debug, Display, Error)]
pub struct InvalidFileName;
use std::{fs, path::PathBuf, str::FromStr};

use anyhow::Result;
use arc_swap::{ArcSwap, ArcSwapAny};
use clap::{Args, Parser, Subcommand};
use config::ConfigBuilder;
use convert_case::{Case, Casing};
use derive_more::Display;
use dirs::config_dir;
use figment::{providers::Env, Figment};
use serde::Deserialize;
use thiserror::Error;

use crate::{governor::get_gov, util::LockedMap};

pub struct Config {
    user_dir: PathBuf,
    configs: LockedMap<Box<str>, config::Config>
}
#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    #[arg(short, long, num_args = 0.., group= "v")]
    plugin: Vec<Vec<String>>
}

// #[derive(Debug, Clone, Args)]
// struct PluginArgs {

// }

impl Config {
    pub fn new() -> Self {
        let user_dir = config_dir()
            .ok_or(NoConfigDir)
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
                .filter(|entry| entry.path().extension().map(|extension| extension == "toml").unwrap_or(false));
                //.map() //create config for each file and return im::Hashmap from iter and rcu into gov
        { //Mutex start
            let gov = get_gov()?;
            
        } //Mutex end
        Ok(())
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
pub struct NoConfigDir;
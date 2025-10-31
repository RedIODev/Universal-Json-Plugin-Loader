use std::{collections::HashMap, fs, path::PathBuf, str::FromStr};

use anyhow::Result;
use arc_swap::ArcSwap;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDateTime};
use clap::{Args, Parser, arg, command};
use convert_case::{Case, Casing};
use derive_more::Display;
use dirs::config_dir;
use figment::{providers::Env, Figment};
use thiserror::Error;
use toml::Spanned;
use toml::de::DeValue;
use crate::util::ResultFlatten;


use crate::{governor::get_gov, util::LockedMap};

pub struct Config {
    user_dir: PathBuf,
    configs: LockedMap<Box<str>, config::Config>
}
#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    #[arg(short, long("plugin"), num_args = 0.., )]
    plugins: Vec<PluginOption>
}

impl Cli {
    pub fn plugins(&self) -> &[PluginOption] {
        &self.plugins
    }
}

#[derive(Debug, Clone)]
pub struct PluginOption {
    plugin: Box<str>,
    name: Box<str>,
    value: TomlValue,
}

impl PluginOption {

    pub fn value(&self) -> &TomlValue {
        &self.value
    }

    pub fn plugin(&self) -> &str {
        &self.plugin
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub enum TomlValue { // write adapter from DeValue when needed otherwise parse file info into TomlValue using Serde.
    I64(i64),
    F64(f64),
    Bool(bool),
    Char(char),
    String(Box<str>),
    DateTime(DateTime<FixedOffset>),
    Array(Box<[TomlValue]>),
    Table(HashMap<Box<str>, TomlValue>)
    
}

impl TomlValue {
    pub fn new(value: &str) -> Result<Self, ParseError> {
        if let Ok(integer) = value.parse() {
            return Ok(TomlValue::I64(integer));
        }
        if let Ok(float) = value.parse() {
            return Ok(TomlValue::F64(float));
        }
        if let Ok(boolean) = value.parse() {
            return Ok(TomlValue::Bool(boolean));
        }
        if let Ok(character) = value.parse() {
            return Ok(TomlValue::Char(character))
        }
        if let Ok(datetime) = DateTime::<FixedOffset>::parse_from_rfc3339(value) {
            return Ok(TomlValue::DateTime(datetime));
        }
        if value.starts_with('[') && value.ends_with(']') {
            let content = &value[1..value.len()-1];
            return Ok(TomlValue::Array(content.split(',').map(TomlValue::new).collect::<Result<_,_>>()?));
        }
        if value.starts_with('{') && value.ends_with('}') {
            let content = &value[1..value.len()-1];
            return Ok(TomlValue::Table(content.split(',').map(|item| 
                item.split_once('=')
                    .map(|(key, value)| Ok((key.into(), TomlValue::new(value)?)))
                    .ok_or(ParseError::MalformedValue)
                    .flatten_()
                ).collect::<Result<HashMap<Box<str>, TomlValue>, ParseError>>()?));
        }
        Ok(TomlValue::String(value.into()))
    }
}
 
impl FromStr for PluginOption {
    type Err = ParseError;
 
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (plugin, option) = s.split_once(':').ok_or(ParseError::NoPluginPrefix)?;
        let (option_name, option_value) = option.split_once('=').ok_or(ParseError::NoValue)?;
        Ok(PluginOption { plugin: plugin.into(), name: option_name.into(), value: TomlValue::new(option_value)?})
    }
}

#[derive(Debug, Error, Display)]
pub enum ParseError {
    NoPluginPrefix,
    NoValue,
    MalformedValue
}

#[derive(Debug, Clone, Args)]
struct PluginArgs {
    plugin_args: Vec<String>
}

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
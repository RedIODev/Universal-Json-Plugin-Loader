use std::collections::HashMap;
use std::str::FromStr;

use derive_more::Display;
use im::Vector;
use thiserror::Error;
use toml::Value as TomlValue;
use toml::Table;
use clap::Parser;

use crate::util::LockedVec;
use crate::util::ResultFlatten;

#[derive(Parser, Debug)]
#[command(version)]
pub struct CliParser {
    #[arg(short, long("plugin"), num_args = 0.., )]
    plugins: Vec<CliPluginOption>
}

pub struct Cli {
    plugins: LockedVec<CliPluginOption>
}

impl From<CliParser> for Cli {
    fn from(value: CliParser) -> Self {
        Cli { plugins: LockedVec::from_pointee(Vector::from(value.plugins)) }
    }
}

impl Cli {
    pub fn plugins(&self) -> &LockedVec<CliPluginOption> {
        &self.plugins
    }
}


#[derive(Debug, Clone)]
pub struct CliPluginOption {
    plugin_name: Box<str>,
    key: Box<str>,
    value: TomlValue
}




    fn parse_cli(value: &str) -> Result<TomlValue, ParseError> {
        if let Ok(integer) = value.parse() {
            return Ok(TomlValue::Integer(integer));
        }
        if let Ok(float) = value.parse() {
            return Ok(TomlValue::Float(float));
        }
        if let Ok(boolean) = value.parse() {
            return Ok(TomlValue::Boolean(boolean));
        }
        if let Ok(datetime) = toml::value::Datetime::from_str(value) {
            return Ok(TomlValue::Datetime(datetime));
        }
        if value.starts_with('[') && value.ends_with(']') {
            let content = &value[1..value.len()-1];
            return Ok(TomlValue::Array(content.split(',').map(parse_cli).collect::<Result<_,_>>()?));
        }
        if value.starts_with('{') && value.ends_with('}') {
            let content = &value[1..value.len()-1];
            return Ok(TomlValue::Table(content.split(',').map(|item| 
                item.split_once('=')
                    .map(|(key, value)| Ok((key.into(), parse_cli(value)?)))
                    .ok_or(ParseError::MalformedValue)
                    .flatten_()
                ).collect::<Result<toml::map::Map<String, TomlValue>, ParseError>>()?));
        }
        Ok(TomlValue::String(value.into()))
    }

impl CliPluginOption {
    /// In case an multiple options with the same plugin and key are given. Only the last one is inserted. 
    pub fn join_table<'a>(options: impl IntoIterator<Item =&'a CliPluginOption>) -> HashMap<Box<str>, Table> {
        options.into_iter().fold(Default::default(), 
        |mut map, option| {
            map.entry(option.plugin_name.clone())
                .or_default()
                .insert(option.key.to_string(), option.value.clone());
            map
        })
    }
}

impl FromStr for CliPluginOption {
    type Err = ParseError;
 
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (plugin_name, option) = s.split_once(':').ok_or(ParseError::NoPluginPrefix)?;
        let (option_key, option_value) = option.split_once('=').ok_or(ParseError::NoValue)?;
        Ok(CliPluginOption { plugin_name: plugin_name.into(), key: option_key.into(), value: parse_cli(option_value)?})
    }
}

#[derive(Debug, Error, Display)]
pub enum ParseError {
    NoPluginPrefix,
    NoValue,
    MalformedValue
}
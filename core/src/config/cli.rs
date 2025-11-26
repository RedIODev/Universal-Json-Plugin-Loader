use std::collections::HashMap;
use core::str::FromStr;

use clap::Parser;
use derive_more::Display;
use im::Vector;
use thiserror::Error;
use toml::Table;
use toml::Value as TomlValue;

use crate::util::LockedVec;
use crate::util::ResultFlatten as _;

#[derive(Parser, Debug)]
#[command(version)]
pub struct CliParser {
    #[arg(short, long("plugin"), num_args = 0.., )]
    plugins: Vec<PluginOption>,
}

pub struct Cli {
    plugins: LockedVec<PluginOption>,
}

impl From<CliParser> for Cli {
    fn from(value: CliParser) -> Self {
        Cli {
            plugins: LockedVec::from_pointee(Vector::from(value.plugins)),
        }
    }
}

impl Cli {
    pub fn plugins(&self) -> &LockedVec<PluginOption> {
        &self.plugins
    }
}

#[derive(Debug, Clone)]
pub struct PluginOption {
    plugin_name: Box<str>,
    key: Box<str>,
    value: TomlValue,
}

fn parse_cli(value: &str) -> Result<TomlValue, CliError> {
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
        let content = &value[1..value.len() - 1];
        return Ok(TomlValue::Array(
            content
                .split(',')
                .map(parse_cli)
                .collect::<Result<_, _>>()?,
        ));
    }
    if value.starts_with('{') && value.ends_with('}') {
        let content = &value[1..value.len() - 1];
        return Ok(TomlValue::Table(
            content
                .split(',')
                .map(|item| {
                    item.split_once('=')
                        .map(|(key, value)| Ok((key.into(), parse_cli(value)?)))
                        .ok_or(CliError::MalformedValue)
                        .flatten_()
                })
                .collect::<Result<toml::map::Map<String, TomlValue>, CliError>>()?,
        ));
    }
    Ok(TomlValue::String(value.into()))
}

impl PluginOption {
    /// In case an multiple options with the same plugin and key are given. Only the last one is inserted.
    pub fn join_table<'a>(
        options: impl IntoIterator<Item = &'a PluginOption>,
    ) -> HashMap<Box<str>, Table> {
        options
            .into_iter()
            .fold(HashMap::default(), |mut map, option| {
                map.entry(option.plugin_name.clone())
                    .or_default()
                    .insert(option.key.to_string(), option.value.clone());
                map
            })
    }
}

impl FromStr for PluginOption {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (plugin_name, option) = s.split_once(':').ok_or(CliError::NoPluginPrefix)?;
        let (option_key, option_value) = option.split_once('=').ok_or(CliError::NoValue)?;
        Ok(PluginOption {
            plugin_name: plugin_name.into(),
            key: option_key.into(),
            value: parse_cli(option_value)?,
        })
    }
}

#[derive(Debug, Error, Display)]
pub enum CliError {
    NoPluginPrefix,
    NoValue,
    MalformedValue,
}

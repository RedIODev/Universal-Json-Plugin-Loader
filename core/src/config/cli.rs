use core::str::FromStr;
use std::collections::HashMap;

use clap::Parser as ClapParser;
use derive_more::Display;
use im::Vector;
use thiserror::Error;
use toml::Table;
use toml::Value as TomlValue;
use toml::value::Datetime;
use toml::map::Map as TomlMap;

use crate::util::LockedVec;
use crate::util::ResultFlatten as _;

#[derive(ClapParser, Debug)]
#[command(version)]
pub struct Parser {
    #[arg(short, long("plugin"), num_args = 0.., )]
    plugins: Vec<PluginOption>,
}

pub struct Cli {
    plugins: LockedVec<PluginOption>,
}

impl From<Parser> for Cli {
    fn from(value: Parser) -> Self {
        Self {
            plugins: LockedVec::from_pointee(Vector::from(value.plugins)),
        }
    }
}

impl Cli {
    pub const fn plugins(&self) -> &LockedVec<PluginOption> {
        &self.plugins
    }
}

#[derive(Debug, Clone)]
pub struct PluginOption {
    key: Box<str>,
    plugin_name: Box<str>,
    value: TomlValue,
}


impl PluginOption {
    /// In case an multiple options with the same plugin and key are given. Only the last one is inserted.
    pub fn join_table<'item, I: IntoIterator<Item = &'item Self>>(
        options: I,
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
        Ok(Self {
            plugin_name: plugin_name.into(),
            key: option_key.into(),
            value: parse_cli(option_value)?,
        })
    }
}

#[expect(clippy::module_name_repetitions, reason = "Error enums should be named after their modules")]
#[derive(Debug, Error, Display)]
pub enum CliError {
    MalformedValue,
    NoPluginPrefix,
    NoValue,
}


fn parse_cli(source_value: &str) -> Result<TomlValue, CliError> {
    if let Ok(integer) = source_value.parse() {
        return Ok(TomlValue::Integer(integer));
    }
    if let Ok(float) = source_value.parse() {
        return Ok(TomlValue::Float(float));
    }
    if let Ok(boolean) = source_value.parse() {
        return Ok(TomlValue::Boolean(boolean));
    }
    if let Ok(datetime) = Datetime::from_str(source_value) {
        return Ok(TomlValue::Datetime(datetime));
    }
    if source_value.starts_with('[') && source_value.ends_with(']') {
        let content = source_value.get(1..source_value.len() - 1).ok_or(CliError::MalformedValue)?;
        return Ok(TomlValue::Array(
            content
                .split(',')
                .map(parse_cli)
                .collect::<Result<_, _>>()?,
        ));
    }
    if source_value.starts_with('{') && source_value.ends_with('}') {
        let content = source_value.get(1..source_value.len() - 1).ok_or(CliError::MalformedValue)?;
        return Ok(TomlValue::Table(
            content
                .split(',')
                .map(|item| {
                    item.split_once('=')
                        .map(|(key, value)| Ok((key.into(), parse_cli(value)?)))
                        .ok_or(CliError::MalformedValue)
                        .flatten_()
                })
                .collect::<Result<TomlMap<String, TomlValue>, CliError>>()?,
        ));
    }
    Ok(TomlValue::String(source_value.into()))
}


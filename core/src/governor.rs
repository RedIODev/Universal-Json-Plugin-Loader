use std::{marker::PhantomData, sync::{Arc, LazyLock}};

use crate::{
    GOV, config::{Config, cli::{Cli, CliParser}}, loader::Loader, runtime::{
        Runtime, endpoint::{Endpoint, Endpoints, register_core_endpoints}, event::{Event, Events, register_core_events}
    }, util::{GuardExt, LockedMap, MappedGuard}
};
use anyhow::Result;
use arc_swap::ArcSwap;
use clap::Parser;
use derive_more::Display;
use thiserror::Error;

pub struct Governor {
    loader: Loader,
    events: LockedMap<Box<str>, Event>,
    endpoints: LockedMap<Box<str>, Endpoint>,
    runtime: Runtime,
    config: Config,
    cli: LazyLock<Cli>
}

pub type GovernorReadGuard = MappedGuard<Option<Arc<Governor>>, Arc<Governor>>;

pub fn get_gov() -> Result<GovernorReadGuard, GovernorError> {
    GOV.load()
        .try_map(|g| g.as_ref().map(Arc::clone).ok_or(GovernorError))
}

#[derive(Error, Debug, Display, Clone)]
pub(crate) struct GovernorError;

///
/// Should only be created in main of this binary once.
/// Ensures that the globally shared state is destroyed when the program ends.
pub(super) struct GovernorLifetime(PhantomData<()>);

impl GovernorLifetime {
    pub(super) fn new() -> Result<Self> {
        GOV.rcu(|_| Some(Arc::new(Governor::new())));
        Ok(Self(PhantomData))
    }
}

impl Drop for GovernorLifetime {
    fn drop(&mut self) {
        Runtime::shutdown();
    }
}

impl Governor {
    pub fn new() -> Self {
        let runtime = Runtime::new();
        let gov = Self {
            loader: Loader::new(),
            events: ArcSwap::default(),
            endpoints: ArcSwap::default(),
            runtime,
            config: Config::new(),
            cli: LazyLock::new(|| CliParser::parse().into())
        };
        register_core_endpoints(gov.endpoints(), gov.runtime().core_id());
        register_core_events(gov.events(), gov.runtime().core_id());
        gov
    }

    pub fn events(&self) -> &Events {
        &self.events
    }

    pub fn endpoints(&self) -> &Endpoints {
        &self.endpoints
    }

    pub fn loader(&self) -> &Loader {
        &self.loader
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn cli(&self) -> &Cli {
        &self.cli
    }
}

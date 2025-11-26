extern crate alloc;

use core::{marker::PhantomData};


use alloc::sync::Arc;
use crate::{
    config::{Config, cli::{Cli, CliParser}}, loader::Loader, runtime::{
        Runtime, endpoint::{Endpoint, Endpoints, register_core_endpoints}, event::{Event, Events, register_core_events}
    }, util::{GuardExt, LazyInit, LockedMap, MappedGuard}
};
use arc_swap::{ArcSwap, ArcSwapOption};
use clap::Parser;
use derive_more::Display;
use thiserror::Error;

pub struct Governor {
    loader: Loader,
    events: LockedMap<Box<str>, Event>,
    endpoints: LockedMap<Box<str>, Endpoint>,
    runtime: Runtime,
    config: Config,
    cli: LazyInit<Cli>
}

pub type GovernorReadGuard = MappedGuard<Option<Arc<Governor>>, Arc<Governor>>;

pub fn get_gov() -> Result<GovernorReadGuard, GovernorError> {
    GOV.load()
        .try_map(|g| g.as_ref().map(Arc::clone).ok_or(GovernorError))
}

pub (super) static GOV: ArcSwapOption<Governor> = ArcSwapOption::const_empty();

///
/// Should only be created in main of this binary once.
/// Ensures that the globally shared state is destroyed when the program ends.
pub(super) struct GovernorLifetime(PhantomData<()>);

impl GovernorLifetime {
    pub(super) fn new() -> Self {
        GOV.rcu(|_| Some(Arc::default()));
        Self(PhantomData)
    }
}

impl Drop for GovernorLifetime {
    fn drop(&mut self) {
        Runtime::shutdown();
    }
}

impl Default for Governor {
    fn default() -> Self {
        let runtime = Runtime::default();
        let gov = Self {
            loader: Loader::default(),
            events: ArcSwap::default(),
            endpoints: ArcSwap::default(),
            runtime,
            config: Config::default(),
            cli: LazyInit::new(|| CliParser::parse().into())
        };
        register_core_endpoints(gov.endpoints(), gov.runtime().core_id());
        register_core_events(gov.events(), gov.runtime().core_id());
        gov 
    }
}

impl Governor {

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
        self.cli.get()
    }
}

#[derive(Error, Debug, Display, Clone)]
pub struct GovernorError;
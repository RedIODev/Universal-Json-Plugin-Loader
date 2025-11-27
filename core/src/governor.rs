use core::marker::PhantomData;

use crate::{
    config::{
        Config,
        cli::{Cli, Parser},
    },
    loader::Loader,
    runtime::{
        Runtime,
        endpoint::{Endpoint, Endpoints, register_core_endpoints},
        event::{Event, Events, register_core_events},
    },
    util::{GuardExt as _, LazyInit, LockedMap, MappedGuard},
};
use alloc::sync::Arc;
use arc_swap::{ArcSwap, ArcSwapOption};
use clap::Parser as _;
use derive_more::Display;
use thiserror::Error;

pub static GOV: ArcSwapOption<Governor> = ArcSwapOption::const_empty();

pub struct Governor {
    cli: LazyInit<Cli>,
    config: Config,
    endpoints: LockedMap<Box<str>, Endpoint>,
    events: LockedMap<Box<str>, Event>,
    loader: Loader,
    runtime: Runtime,
}

///
/// Should only be created in main of this binary once.
/// Ensures that the globally shared state is destroyed when the program ends.
pub struct GovernorLifetime(PhantomData<()>);

impl GovernorLifetime {
    #[expect(clippy::single_call_fn, reason = "A new governor lifetime should only be created at 1 location")]
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

pub type GovernorReadGuard = MappedGuard<Option<Arc<Governor>>, Arc<Governor>>;




impl Default for Governor {
    #[expect(clippy::expect_used, reason = "Not registering core is not recoverable")]
    fn default() -> Self {
        let runtime = Runtime::default();
        let gov = Self {
            loader: Loader::default(),
            events: ArcSwap::default(),
            endpoints: ArcSwap::default(),
            runtime,
            config: Config::default(),
            cli: LazyInit::new(|| Parser::parse().into()),
        };
        register_core_endpoints(gov.endpoints(), gov.runtime().core_id()).expect("Fatal Error: Registering of core endpoints failed!");
        register_core_events(gov.events(), gov.runtime().core_id()).expect("Fatal Error: Registering of core events failed!");
        gov
    }
}

impl Governor {
    pub fn cli(&self) -> &Cli {
        self.cli.get()
    }
    
    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub const fn endpoints(&self) -> &Endpoints {
        &self.endpoints
    }

    pub const fn events(&self) -> &Events {
        &self.events
    }

    pub const fn loader(&self) -> &Loader {
        &self.loader
    }
    
    pub const fn runtime(&self) -> &Runtime {
        &self.runtime
    }

}

#[derive(Error, Debug, Display, Clone)]
pub struct GovernorError;

pub fn get_gov() -> Result<GovernorReadGuard, GovernorError> {
    GOV.load()
        .try_map(|gov| gov.as_ref().map(Arc::clone).ok_or(GovernorError))
}
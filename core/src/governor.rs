use std::{collections::HashMap, marker::PhantomData};

use anyhow::Result;
use derive_more::Display;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLockReadGuard, RwLockWriteGuard};
use thiserror::Error;
use crate::{loader::Loader, runtime::{endpoint::{register_core_endpoints, Endpoint}, event::{register_core_events, Event}, Runtime}, GGL};

pub type Events = HashMap<Box<str>, Event>;
pub type Endpoints = HashMap<Box<str>, Endpoint>;

pub struct Governor {
    loader: Loader,
    events: Events,
    endpoints: Endpoints,
    runtime: Runtime
}

pub fn read_gov<'a>() -> Result<MappedRwLockReadGuard<'a, Governor>> {
    Ok(RwLockReadGuard::try_map(GGL.read(), Option::as_ref)
        .map_err(|_| GovernorError)?)
}

pub fn write_gov<'a>() -> Result<MappedRwLockWriteGuard<'a, Governor>> {
    Ok(RwLockWriteGuard::try_map(GGL.write(), Option::as_mut)
         .map_err(|_| GovernorError)?)
}

#[derive(Error, Debug, Display)]
pub (crate) struct GovernorError;

pub struct GovernorLifetime(PhantomData<()>);

impl GovernorLifetime {
    pub fn new() -> Result<Self> {
        GGL.write().get_or_insert_with(|| Governor::new());
        Ok(Self(PhantomData))
    }
}



impl Drop for GovernorLifetime {
    fn drop(&mut self) {
        let gov = GGL.write().take();
        match gov {
            Some(g) => drop(g),
            None => panic!("GGL was dead on shutdown. Unreachable!")
        }
    }
}

impl Governor {
    pub fn new() -> Self {
        let runtime = Runtime::new();
        Self { loader: Loader::new(), events: register_core_events(runtime.core_id()), endpoints: register_core_endpoints(runtime.core_id()), runtime }
    }

    pub fn events(&self) -> &Events {
        &self.events
    }

    pub fn events_mut(&mut self) -> &mut Events {
        &mut self.events
    }

    pub fn endpoints(&self) -> &Endpoints {
        &self.endpoints
    }

    pub fn endpoints_mut(&mut self) -> &mut Endpoints {
        &mut self.endpoints
    }

    pub fn loader(&self) -> &Loader {
        &self.loader
    }

    pub fn loader_mut(&mut self) -> &mut Loader {
        &mut self.loader
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut Runtime {
        &mut self.runtime
    }
}

impl Drop for Governor {
    fn drop(&mut self) {
        todo!()
    }
}
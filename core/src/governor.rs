use std::collections::HashMap;

use finance_together_api::cbindings::CUuid;

use crate::{loader::{Loader, Plugin}, runtime::{endpoint::{register_core_endpoints, Endpoint}, event::{register_core_events, Event}, Runtime}};

pub type Events = HashMap<Box<str>, Event>;
pub type Endpoints = HashMap<Box<str>, Endpoint>;

pub struct Governor {
    loader: Loader,
    events: Events,
    endpoints: Endpoints,
    runtime: Runtime
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

    pub fn core_id(&self) -> CUuid {
        self.runtime.core_id()
    }

    pub fn plugins_mut(&mut self) -> &mut HashMap<CUuid, Plugin> {
        self.loader.plugins_mut()
    }

    pub fn plugins(&self) -> &HashMap<CUuid, Plugin> {
        self.loader.plugins()
    }
}
use std::collections::HashMap;


use crate::{loader::Loader, runtime::{endpoint::{register_core_endpoints, Endpoint}, event::{register_core_events, Event}, Runtime}};

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

    // pub fn core_id(&self) -> CUuid {
    //     self.runtime.core_id()
    // }

    // pub fn cancel_power(&mut self) {
    //     self.runtime.cancel_power();
    // }

    // pub fn is_power_canceled(&self) -> bool {
    //     self.runtime.is_power_canceled()
    // }

    // pub fn set_main_handle(&mut self, main_handle: Thread) {
    //     self.runtime.set_main_handle(main_handle);
    // }

    // pub fn plugins_mut(&mut self) -> &mut HashMap<CUuid, Plugin> {
    //     self.loader.plugins_mut()
    // }

    // pub fn plugins(&self) -> &HashMap<CUuid, Plugin> {
    //     self.loader.plugins()
    // }
}
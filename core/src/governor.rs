use std::collections::HashMap;

use finance_together_api::cbindings::CUuid;
use libloading::Library;

use crate::{loader::Loader, runtime::{event::{register_core_events, Event}, Runtime}};

pub type Events = HashMap<Box<str>, Event>;

pub struct Governor {
    loader: Loader,
    events: Events,
    runtime: Runtime
}

impl Governor {
    pub fn new() -> Self {
        let runtime = Runtime::new();
        Self { loader: Loader::new(), events: register_core_events(runtime.core_id()), runtime }
    }

    pub fn events(&self) -> &Events {
        &self.events
    }

    pub fn events_mut(&mut self) -> &mut Events {
        &mut self.events
    }

    pub fn core_id(&self) -> CUuid {
        self.runtime.core_id()
    }

    pub fn libs_mut(&mut self) -> &mut Vec<Library> {
        self.loader.libs_mut()
    }
}
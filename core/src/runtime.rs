use std::{collections::HashSet, hash::Hash};

use finance_together_api::{cbindings::{CHandler, CHandlerFP, CString, CUuid, ServiceError}, Handler};
use jsonschema::Validator;
use uuid::Uuid;

use crate::LOADER;


struct Runtime {

}


unsafe extern "C" fn handler_register(handler_fp: CHandlerFP, plugin_id: CUuid, event_name: CString) -> CHandler {
    let Some(function) = handler_fp else {
        return CHandler::new_error(ServiceError::InvalidInput0);
    };
    let Ok(mut events) = LOADER.handler.lock() else {
        return CHandler::new_error(ServiceError::CoreInternalError);
    };
    let Ok(event_name) = event_name.as_str() else {
        return CHandler::new_error(ServiceError::InvalidInput2);
    };
    let Some(event) = events.get_mut(event_name) else {
        return CHandler::new_error(ServiceError::NotFound);
    };

    let handler = Handler { function, handler_id: CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair())};
    if !event.handlers.insert(StoredHandler { handler: handler.clone(), plugin_id }) {
        return  CHandler::new_error(ServiceError::Duplicate);
    }
    handler.into()
}

unsafe extern "C" fn handler_unregister(handler_id: CUuid, plugin_id: CUuid, event_name: CString) -> ServiceError {
    let Ok(mut events) = LOADER.handler.lock() else {
        return  ServiceError::CoreInternalError;
    };
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput2;
    };
    let Some(event) = events.get_mut(event_name) else {
        return  ServiceError::NotFound;
    };
    let Some(handler) = event.handlers.iter().find(|h| h.handler.handler_id == handler_id) else {
        return ServiceError::NotFound;
    };
    if handler.plugin_id != plugin_id {
        return  ServiceError::Unauthorized;
    }

    event.handlers.remove(&handler.clone());
    ServiceError::Success
}


pub(crate) struct Event {
    pub(crate) handlers: HashSet<StoredHandler>,
    argument_validator: Validator,
    plugin_id: CUuid
}

impl Event {
    pub fn new(argument_validator: Validator, plugin_id: CUuid) -> Self {
        Self { handlers: HashSet::new(), argument_validator, plugin_id }
    }
}


#[derive(Clone)]
pub(crate) struct StoredHandler {
    handler: Handler,
    plugin_id: CUuid
}

impl Hash for StoredHandler {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.plugin_id.hash(state);
    }
}

impl PartialEq for StoredHandler {
    fn eq(&self, other: &Self) -> bool {
        self.plugin_id == other.plugin_id
    }
}

impl Eq for StoredHandler {}

impl StoredHandler {
    pub fn new(handler: Handler, plugin_id: CUuid) -> Self {
        Self { handler, plugin_id }
    }
}

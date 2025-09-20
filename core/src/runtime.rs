use std::{collections::{hash_map::Entry, HashMap, HashSet}, hash::Hash};

use finance_together_api::{cbindings::{ApplicationContext, CHandler, CHandlerFP, CString, CUuid, ServiceError}, Handler};
use jsonschema::Validator;
use serde_json::json;
use uuid::Uuid;

use crate::LOADER;


pub struct Runtime {
    core_id: CUuid
}

impl Runtime {
    
    pub fn new() -> Self {
        Self { core_id: CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair()) }
    }

    pub fn init(&self) {
        
    }


    pub(crate) fn register_core_events(&self) -> HashMap<Box<str>, Event> {
        
        let mut hashmap = HashMap::new();
        hashmap.insert("core:init".into(), Event::new(Runtime::schema_from_file(include_str!("../event/init.json")),self.core_id));
        
        hashmap
    }

    fn schema_from_file(file:&str) -> Validator {
        jsonschema::validator_for(&json!(file)).expect("invalid core schema!")

    }
}

unsafe extern "C" fn context_supplier() -> ApplicationContext {
    ApplicationContext { 
        handlerRegisterService: Some(handler_register), 
        HandlerUnregisterService: Some(handler_unregister), 
        eventRegisterService: Some(event_register), 
        EventUnregisterService: Some(event_unregister), 
        eventTriggerService: Some(event_trigger) 
    }
}

unsafe extern "C" fn handler_register(handler_fp: CHandlerFP, plugin_id: CUuid, event_name: CString) -> CHandler {
    let Some(function) = handler_fp else {
        return CHandler::new_error(ServiceError::InvalidInput0);
    };
    let Ok(mut loader) = LOADER.lock() else {
        return CHandler::new_error(ServiceError::CoreInternalError);
    };
    let events = &mut loader.events;
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
    let Ok(mut loader) = LOADER.lock() else {
        return  ServiceError::CoreInternalError;
    };
    let events = &mut loader.events;
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

unsafe extern "C" fn event_register(argument_schema: CString, plugin_id: CUuid, event_name: CString) -> ServiceError {
    let Ok(mut loader) = LOADER.lock() else {
        return  ServiceError::CoreInternalError;
    };
    let events = &mut loader.events;
    let Ok(argument_schema) = argument_schema.as_str() else {
        return ServiceError::InvalidInput0;
    };
    let Ok(validator) = jsonschema::validator_for(&json!(argument_schema)) else {
        return ServiceError::InvalidInput0;
    };
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput2;
    };
    let event = Event::new(validator, plugin_id);
    if events.contains_key(event_name) {
        return  ServiceError::Duplicate;
    }
    events.insert(event_name.into(), event);
    ServiceError::Success
}

unsafe extern "C" fn event_unregister(plugin_id: CUuid, event_name: CString) -> ServiceError {
    let Ok(mut loader) = LOADER.lock() else {
        return ServiceError::CoreInternalError;
    };
    let events = &mut loader.events;
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput1;
    };
    let Entry::Occupied(o) = events.entry(event_name.into()) else {
        return ServiceError::NotFound;
    };
    if o.get().plugin_id != plugin_id {
        return ServiceError::Unauthorized;
    }
    o.remove();
    ServiceError::Success
}

unsafe extern "C" fn event_trigger(plugin_id: CUuid, event_name: CString, arguments: CString) -> ServiceError {
    let Ok(loader) = LOADER.lock() else {
        return ServiceError::CoreInternalError;
    };
    let events = &loader.events;
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput1;
    };
    let Some(event) = events.get(event_name) else {
        return ServiceError::NotFound;
    };
    if event.plugin_id != plugin_id {
        return ServiceError::Unauthorized;
    }
    let Ok(arguments) = arguments.as_str() else {
        return ServiceError::InvalidInput2;
    };
    if let Err(_) = event.argument_validator.validate(&json!(arguments)) {
        return ServiceError::InvalidInput2;
    }
    for handler in &event.handlers {
        let function = handler.handler.function;
        unsafe { function(Some(context_supplier), CString::from_string(arguments.to_string()))}
    }
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

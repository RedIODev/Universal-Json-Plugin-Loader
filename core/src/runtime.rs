use std::{collections::{hash_map::Entry, HashMap, HashSet}, hash::Hash};

use finance_together_api::{cbindings::{ApplicationContext, CHandler, CHandlerFP, CString, CUuid, ServiceError}, Handler};
use jsonschema::Validator;
use serde_json::json;
use uuid::Uuid;

use crate::GGL;


pub struct Runtime {
    core_id: CUuid
}

impl Runtime {
    
    pub fn new() -> Self {
        Self { core_id: CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair()) }
    }

    pub fn init() -> Result<(), ()> { //make ServiceError Error compatible
        let core_id = GGL.read().unwrap().core_id();
        let result = unsafe { event_trigger(core_id, "core:init".into(), json!({"version": "0.1.0"}).to_string().into()) };
        if result == ServiceError::Success {
            return Ok(());
        }
        Err(())
    }

    pub fn core_id(&self) -> CUuid {
        self.core_id
    }


    pub fn register_core_events(&self) -> HashMap<Box<str>, Event> {
        
        let mut hashmap = HashMap::new();
        hashmap.insert("core:init".into(), Event::new(Runtime::schema_from_file(include_str!("../event/init.json")),self.core_id));
        hashmap.insert("core:event".into(), Event::new(Runtime::schema_from_file(include_str!("../event/event.json")), self.core_id));
        hashmap
    }

    fn schema_from_file(file:&str) -> Validator {
        jsonschema::validator_for(&serde_json::from_str(file).expect("invalid json!")).expect("invalid core schema!")
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
    let Ok(event_name) = event_name.as_str() else {
        return CHandler::new_error(ServiceError::InvalidInput2);
    };
    let handler = Handler { function, handler_id: CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair())};
    { // Mutex start
        let Ok(mut gov) = GGL.write() else {
            return CHandler::new_error(ServiceError::CoreInternalError);
        };
        let Some(event) = gov.events_mut().get_mut(event_name) else {
            return CHandler::new_error(ServiceError::NotFound);
        };
        
        if !event.handlers.insert(StoredHandler { handler: handler.clone(), plugin_id }) {
            return  CHandler::new_error(ServiceError::Duplicate);
        }
    } // Mutex end

    handler.into()
}

unsafe extern "C" fn handler_unregister(handler_id: CUuid, plugin_id: CUuid, event_name: CString) -> ServiceError {
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput2;
    };
    { // Mutex start
        let Ok(mut gov) = GGL.write() else {
            return  ServiceError::CoreInternalError;
        };
        let Some(event) = gov.events_mut().get_mut(event_name) else {
            return  ServiceError::NotFound;
        };
        let Some(handler) = event.handlers.iter().find(|h| h.handler.handler_id == handler_id) else {
            return ServiceError::NotFound;
        };
        if handler.plugin_id != plugin_id {
            return  ServiceError::Unauthorized;
        }
        event.handlers.remove(&handler.clone());
    } // Mutex end

    ServiceError::Success
}

unsafe extern "C" fn event_register(argument_schema: CString, plugin_id: CUuid, event_name: CString) -> ServiceError {
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput2;
    };
    let Ok(argument_schema) = argument_schema.as_str() else {
        return ServiceError::InvalidInput0;
    };
    let Ok(argument_schema) = serde_json::from_str(argument_schema) else {
        return ServiceError::InvalidInput0;
    };
    let Ok(validator) = jsonschema::validator_for(&argument_schema) else {
        return ServiceError::InvalidInput0;
    };
    let event = Event::new(validator, plugin_id);
    
    let core_id = { // Mutex start
        let Ok(mut gov) = GGL.write() else {
            return ServiceError::CoreInternalError;
        };
        let events = gov.events_mut();
        if events.contains_key(event_name) {
            return ServiceError::Duplicate;
        }
        events.insert(event_name.into(), event);
        gov.core_id()
    }; // Mutex end
    unsafe { event_trigger(core_id, "core:event".into(), format!("").into()) }; // locks mutex
    ServiceError::Success
}

unsafe extern "C" fn event_unregister(plugin_id: CUuid, event_name: CString) -> ServiceError {
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput1;
    };
    { // Mutex start
        let Ok(mut gov) = GGL.write() else {
            return ServiceError::CoreInternalError;
        };
        let Entry::Occupied(o) = gov.events_mut().entry(event_name.into()) else {
            return ServiceError::NotFound;
        };
        if o.get().plugin_id != plugin_id {
            return ServiceError::Unauthorized;
        }
        o.remove();

    } // Mutex end
    ServiceError::Success
}

unsafe extern "C" fn event_trigger(plugin_id: CUuid, event_name: CString, arguments: CString) -> ServiceError {
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput1;
    };
    let Ok(arguments) = arguments.as_str() else {
        return ServiceError::InvalidInput2;
    };
    let funcs: Vec<_> = { // Mutex start
        let Ok(gov) = GGL.read() else {
            return ServiceError::CoreInternalError;
        };
        let Some(event) = gov.events().get(event_name) else {
            return ServiceError::NotFound;
        };
        if event.plugin_id != plugin_id {
            return ServiceError::Unauthorized;
        }
        let Ok(arguments) = serde_json::from_str(arguments) else {
            return  ServiceError::InvalidInput2;
        };
        if let Err(_) = event.argument_validator.validate(&arguments) {
            return ServiceError::InvalidInput2;
        }
        event.handlers.iter().map(|h|h.handler.function).collect()
    }; // Mutex end
    for func in funcs {
        unsafe { func(Some(context_supplier), arguments.into())} // might lock mutex
    }
    ServiceError::Success
}


pub struct Event {
    pub handlers: HashSet<StoredHandler>,
    argument_validator: Validator,
    plugin_id: CUuid
}

impl Event {
    pub fn new(argument_validator: Validator, plugin_id: CUuid) -> Self {
        Self { handlers: HashSet::new(), argument_validator, plugin_id }
    }
}


#[derive(Clone)]
pub struct StoredHandler {
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

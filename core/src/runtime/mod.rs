pub mod event;
pub mod endpoint;

use finance_together_api::cbindings::{ApplicationContext, CUuid};
use anyhow::Result;
use serde_json::json;
use uuid::Uuid;

use crate::{runtime::{endpoint::{endpoint_register, endpoint_request, endpoint_unregister}, event::{event_register, event_trigger, event_unregister, handler_register, handler_unregister}}, GGL};


pub struct Runtime {
    core_id: CUuid
}

impl Runtime {
    
    pub fn new() -> Self {
        Self { core_id: CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair()) }
    }

    pub fn init() -> Result<()> {
        let core_id = GGL.read().unwrap().core_id();
        unsafe { event_trigger(core_id, "core:init".into(), json!({"version": "0.1.0"}).to_string().into()) }.result()?;
        Ok(())
    }

    pub fn core_id(&self) -> CUuid {
        self.core_id
    }
}

unsafe extern "C" fn context_supplier() -> ApplicationContext {
    ApplicationContext { 
        handlerRegisterService: Some(handler_register), 
        HandlerUnregisterService: Some(handler_unregister), 
        eventRegisterService: Some(event_register), 
        eventUnregisterService: Some(event_unregister), 
        eventTriggerService: Some(event_trigger),
        endpointRegisterService: Some(endpoint_register),
        endpointUnregisterService: Some(endpoint_unregister),
        endpointRequestService: Some(endpoint_request) 
    }
}



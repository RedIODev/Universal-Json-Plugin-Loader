pub mod endpoint;
pub mod event;

use std::thread::Thread;

use anyhow::Result;
use derive_more::Display;
use finance_together_api::cbindings::{ApplicationContext, CUuid};
use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    governor::Governor, loader::Loader, runtime::{
        endpoint::{endpoint_register, endpoint_request, endpoint_unregister},
        event::{
            event_register, event_trigger, event_unregister, handler_register, handler_unregister,
        },
    }, GGL
};

#[derive(Deserialize, Serialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PowerState {
    Shutdown,
    Restart,
    Cancel,
}

pub struct Runtime {
    core_id: CUuid,
    power_state: Option<PowerState>,
    main_handle: Thread,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            core_id: CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair()),
            power_state: None,
            main_handle: std::thread::current(),
        }
    }

    pub fn init() -> Result<()> {
        let core_id;
        let plugins;
        {
            // Mutex start
            let gov = GGL.read().map_err(|_| LockError)?;
            core_id = gov.runtime().core_id();
            plugins = gov
                .loader()
                .plugins()
                .values()
                .map(|plugin| json!({"name": *plugin.name, "version": *plugin.version}))
                .collect::<Vec<_>>()
        } // Mutex end

        unsafe {
            event_trigger(
                core_id,
                "core:init".into(),
                json!({"version": "0.1.0", "plugins": plugins})
                    .to_string()
                    .into(),
            )
        }
        .result()?;
        Ok(())
    }

    pub fn restart() -> Result<()> {
        *GGL.write().map_err(|_| LockError)? = Governor::new();
        Loader::load_libraries()?;
        Runtime::init()
    }

    pub fn park() -> Result<Option<PowerState>> {
        std::thread::park();
        {
        // Mutex start
            let mut gov = GGL.write().map_err(|_| LockError)?;
            Ok(gov.runtime_mut().check_and_reset_power())
        } // Mutex end
    }

    pub fn core_id(&self) -> CUuid {
        self.core_id
    }

    pub fn check_and_reset_power(&mut self) -> Option<PowerState> {
        self.power_state.take()
    }

    pub fn set_power(&mut self, power_state: PowerState){
        self.power_state = Some(power_state);
        if power_state != PowerState::Cancel {
            self.main_handle.unpark();
        }
    }
}


fn schema_from_file(file: &str) -> Validator {
    jsonschema::validator_for(&serde_json::from_str(file).expect("invalid json!"))
        .expect("invalid core schema!")
}

#[derive(Error, Debug, Display)]
struct LockError;

unsafe extern "C" fn context_supplier() -> ApplicationContext {
    ApplicationContext {
        handlerRegisterService: Some(handler_register),
        HandlerUnregisterService: Some(handler_unregister),
        eventRegisterService: Some(event_register),
        eventUnregisterService: Some(event_unregister),
        eventTriggerService: Some(event_trigger),
        endpointRegisterService: Some(endpoint_register),
        endpointUnregisterService: Some(endpoint_unregister),
        endpointRequestService: Some(endpoint_request),
    }
}

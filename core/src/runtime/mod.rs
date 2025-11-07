pub mod endpoint;
pub mod event;

use std::{
    num::NonZero,
    sync::{Arc, atomic::Ordering},
    thread::Thread,
};

use crate::{
    config::Config, governor::{get_gov, Governor}, loader::Loader, runtime::{
        endpoint::{endpoint_register, endpoint_request, endpoint_unregister},
        event::{
            event_register, event_trigger, event_unregister, handler_register, handler_unregister,
        },
    }, GOV
};
use anyhow::Result;
use atomic_enum::atomic_enum;
use finance_together_api::cbindings::{ApplicationContext, CUuid};
use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::json;
use threadpool::ThreadPool;
use uuid::Uuid;
use clap::crate_version;

#[atomic_enum]
#[derive(Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PowerState {
    Running,
    Shutdown,
    Restart,
    Cancel,
}

pub struct Runtime {
    core_id: Uuid,
    power_state: AtomicPowerState,
    main_handle: Thread,
    event_pool: ThreadPool,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            core_id: CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair()),
            power_state: AtomicPowerState::new(PowerState::Running),
            main_handle: std::thread::current(),
            event_pool: ThreadPool::new(
                std::thread::available_parallelism()
                    .unwrap_or(NonZero::<usize>::MIN)
                    .into(),
            ),
        }
    }

    pub fn init() -> Result<()> {
        let core_id;
        let plugins;
        {
            // Mutex start
            let gov = get_gov()?;
            core_id = gov.runtime().core_id();
            plugins = gov
                .loader()
                .plugins()
                .load()
                .values()
                .map(|plugin| json!({"name": *plugin.name, "version": *plugin.version}))
                .collect::<Vec<_>>()
        } // Mutex end

        unsafe {
            event_trigger(
                core_id,
                "core:init".into(),
                json!({"version": crate_version!(), "plugins": plugins})
                    .to_string()
                    .into(),
            )
        }
        .result()?;
        Ok(())
    }

    pub fn start() -> Result<()> {
        Config::init()?;
        Loader::load_libraries()?;
        Runtime::init()
    }

    pub fn restart() -> Result<()> {
        if let Some(gov) = &*GOV.load() {
            gov.runtime().event_pool.join();
        }
        GOV.rcu(|_| Some(Arc::new(Governor::new())));
        Runtime::start()
    }

    pub fn shutdown() {
        if let Some(gov) = &*GOV.load() {
            gov.runtime().event_pool.join();
        }
        GOV.rcu(|_| None);
    }

    pub fn park() -> Result<PowerState> {
        std::thread::park();
        {
            // Mutex start
            let gov = get_gov()?;
            Ok(gov.runtime().check_and_reset_power())
        } // Mutex end
    }

    pub fn core_id(&self) -> Uuid {
        self.core_id
    }

    pub fn check_and_reset_power(&self) -> PowerState {
        self.power_state
            .swap(PowerState::Running, Ordering::Relaxed)
    }

    pub fn check_power(&self) -> PowerState {
        self.power_state.load(Ordering::Relaxed)
    }

    pub fn set_power(&self, power_state: PowerState) {
        self.power_state.store(power_state, Ordering::Relaxed);
        if power_state != PowerState::Cancel {
            self.main_handle.unpark();
        }
    }
}

fn schema_from_file(file: &str) -> Validator {
    jsonschema::validator_for(&serde_json::from_str(file).expect("invalid json!"))
        .expect("invalid core schema!")
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
        endpointRequestService: Some(endpoint_request),
    }
}

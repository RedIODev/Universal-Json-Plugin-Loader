pub mod endpoint;
pub mod event;

use std::{
    num::NonZero,
    sync::{Arc, atomic::Ordering},
    thread::Thread,
};

use crate::{
    GOV,
    config::{Config, ConfigError},
    governor::{GovernorError, get_gov},
    loader::{Loader, LoaderError},
    runtime::{
        endpoint::{EndpointRegister, EndpointRequest, EndpointUnregister},
        event::{
            EventHandlerRegister, EventHandlerUnregister, EventRegister, EventTrigger,
            EventUnregister,
        },
    },
};
use atomic_enum::atomic_enum;
use derive_more::Display;
use finance_together_api::{
    ApplicationContext, ServiceError, pointer_traits::{ContextSupplier, EventTriggerService, trait_fn}
};
use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use threadpool::ThreadPool;
use uuid::Uuid;

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

impl Default for Runtime {
    fn default() -> Self {
        Self {
            core_id: Uuid::new_v4(),
            power_state: AtomicPowerState::new(PowerState::Running),
            main_handle: std::thread::current(),
            event_pool: ThreadPool::new(
                std::thread::available_parallelism()
                    .unwrap_or(NonZero::<usize>::MIN)
                    .into(),
            ),
        }
    }
}

impl Runtime {
    pub fn init() -> Result<(), RuntimeError> {
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

        EventTrigger::trigger(
            core_id,
            "core:init",
            json!({"core_version": env!("CARGO_PKG_VERSION"), "plugins": plugins}).to_string(),
        )?;
        Ok(())
    }

    pub fn start() -> Result<(), RuntimeError> {
        Config::init()?;
        Loader::load_libraries()?;
        Runtime::init()
    }

    pub fn restart() -> Result<(), RuntimeError> {
        if let Some(gov) = &*GOV.load() {
            gov.runtime().event_pool.join();
        }
        GOV.rcu(|_| Some(Arc::default()));
        Runtime::start()
    }

    pub fn shutdown() {
        if let Some(gov) = &*GOV.load() {
            gov.runtime().event_pool.join();
        }
        GOV.rcu(|_| None);
    }

    pub fn park() -> Result<PowerState, RuntimeError> {
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

#[trait_fn(ContextSupplier for ContextSupplierImpl)]
fn supply() -> ApplicationContext {
    ApplicationContext::new::<
        EventHandlerRegister,
        EventHandlerUnregister,
        EventRegister,
        EventUnregister,
        EventTrigger,
        EndpointRegister,
        EndpointUnregister,
        EndpointRequest,
    >()
}

#[derive(Debug, Display, Error)]
pub enum RuntimeError {
    Governor(#[from]GovernorError),
    Service(#[from]ServiceError),
    Config(#[from]ConfigError),
    Loader(#[from]LoaderError)
}
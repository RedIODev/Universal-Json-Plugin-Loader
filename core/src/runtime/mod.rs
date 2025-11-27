pub mod endpoint;
pub mod event;

use core::{num::NonZero, sync::atomic::Ordering};
use alloc::sync::Arc;

use std::{
    thread::{self, Thread},
};

use crate::{
    config::{Config, ConfigError},
    governor::{GOV, GovernorError, get_gov},
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
use plugin_loader_api::{
    ApplicationContext, ServiceError, pointer_traits::{ContextSupplier, EventTriggerService as _, trait_fn}
};
use jsonschema::{ValidationError, Validator};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use threadpool::ThreadPool;
use uuid::Uuid;

#[atomic_enum]
#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PowerState {
    Cancel,
    Restart,
    Running,
    Shutdown,
}

pub struct Runtime {
    core_id: Uuid,
    event_pool: ThreadPool,
    main_handle: Thread,
    power_state: AtomicPowerState,
}

#[derive(Debug, Display, Error)]
pub enum RuntimeError {
    Config(#[from]ConfigError),
    Governor(#[from]GovernorError),
    JsonError(#[from]serde_json::Error),
    Loader(#[from]LoaderError),
    Service(#[from]ServiceError),
    ValidatorError(#[from] ValidationError<'static>)
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            core_id: Uuid::new_v4(),
            power_state: AtomicPowerState::new(PowerState::Running),
            main_handle: thread::current(),
            event_pool: ThreadPool::new(
                thread::available_parallelism()
                .unwrap_or(NonZero::<usize>::MIN)
                .into(),
            ),
        }
    }
}

impl Runtime {
    pub fn check_and_reset_power(&self) -> PowerState {
        self.power_state
            .swap(PowerState::Running, Ordering::Relaxed)
    }
    
    pub fn check_power(&self) -> PowerState {
        self.power_state.load(Ordering::Relaxed)
    }
    
    pub const fn core_id(&self) -> Uuid {
        self.core_id
    }
    
    #[expect(clippy::single_call_fn, reason = "function extracted to locate to better module")]
    pub fn init() -> Result<(), RuntimeError> {
        
        let core_id = get_gov()?.runtime().core_id();
        let plugins = get_gov()?
                .loader()
                .plugins()
                .load()
                .values()
                .map(|plugin| json!({"name": *plugin.name(), "version": *plugin.version()}))
                .collect::<Vec<_>>();
        EventTrigger::trigger(
            core_id,
            "core:init",
            json!({"core_version": env!("CARGO_PKG_VERSION"), "plugins": plugins}).to_string(),
        )?;
        Ok(())
    }
    
    #[expect(clippy::single_call_fn, reason = "function extracted to locate to better module")]
    pub fn park() -> Result<PowerState, RuntimeError> {
        thread::park();
        Ok(get_gov()?.runtime().check_and_reset_power())
    }
    
    #[expect(clippy::single_call_fn, reason = "function extracted to locate to better module")]
    pub fn restart() -> Result<(), RuntimeError> {
        let mut old_config_dir = None;
        if let Some(gov) = &*GOV.load() {
            gov.runtime().event_pool.join();
            old_config_dir.clone_from(&Some(Box::from(gov.config().config_dir()?)));
        }
        
        GOV.rcu(|_| Some(Arc::default()));
        if let Some(dir) = old_config_dir {
            Config::set_config_dir(dir)?;
        }
        Self::start()
    }

    pub fn set_power(&self, power_state: PowerState) {
        self.power_state.store(power_state, Ordering::Relaxed);
        if power_state != PowerState::Cancel {
            self.main_handle.unpark();
        }
    }

    #[expect(clippy::single_call_fn, reason = "function extracted to locate to better module")]
    pub fn shutdown() {
        if let Some(gov) = &*GOV.load() {
            gov.runtime().event_pool.join();
        }
        GOV.rcu(|_| None);
    }
    
    pub fn start() -> Result<(), RuntimeError> {
        Config::init()?;
        Loader::load_libraries()?;
        Self::init()
    }
}

fn schema_from_file(file: &str) -> Result<Validator, RuntimeError> {
    Ok(jsonschema::validator_for(&serde_json::from_str(file)?)?)
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

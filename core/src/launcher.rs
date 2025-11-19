use std::path::Path;

use derive_more::Display;
use thiserror::Error;

use crate::{
    config::{Config, ConfigError},
    governor::{GovernorLifetime, get_gov},
    runtime::{PowerState, Runtime, RuntimeError},
};

#[derive(Debug, Clone)]
pub struct Launcher {
    config_path: Box<Path>,
}

impl Launcher {
    pub fn new(config_path: impl AsRef<Path>) -> Self {
        Self {
            config_path: Box::from(config_path.as_ref()),
        }
    }

    pub fn launch(&self) -> Result<(), LaunchError> {
        let gov_lifetime = GovernorLifetime::new();
        Config::set_config_dir(&self.config_path)?;
        Runtime::start()?;
        ctrlc::set_handler(ctrlc_handler)?;
        loop {
            match Runtime::park()? {
                PowerState::Shutdown => break,
                PowerState::Restart => Runtime::restart()?,
                PowerState::Cancel | PowerState::Running => {}
            }
        }
        drop(gov_lifetime); //explicit drop to ensure the GGL lifetime is exactly the lifetime of main.
        Ok(())
    }
}

fn ctrlc_handler() {
    let gov = get_gov().expect("could not acquire governor for shutdown!");
    gov.runtime().set_power(PowerState::Shutdown);
}



#[derive(Debug, Display, Error)]
pub enum LaunchError {
    Runtime(#[from] RuntimeError),
    Ctrlc(#[from] ctrlc::Error),
    Config(#[from] ConfigError),
}

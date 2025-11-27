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

    #[expect(clippy::single_call_fn, reason = "function extracted to locate to better module")]
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: Box::from(config_path.as_ref()),
        }
    }

}

#[derive(Debug, Display, Error)]
pub enum LaunchError {
    Config(#[from] ConfigError),
    Ctrlc(#[from] ctrlc::Error),
    Runtime(#[from] RuntimeError),
}

#[expect(clippy::single_call_fn, reason = "function only used for ctrlc callback")]
#[expect(clippy::expect_used, reason = "cannot recover from signal handler")]
fn ctrlc_handler() {
    let gov = get_gov().expect("could not acquire governor for shutdown!");
    gov.runtime().set_power(PowerState::Shutdown);
}




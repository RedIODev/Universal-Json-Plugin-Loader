#![allow(clippy::missing_errors_doc)]


use arc_swap::ArcSwapOption;
use derive_more::Display;
use thiserror::Error;

use crate::{
    governor::{Governor, GovernorLifetime, get_gov}, runtime::{PowerState, Runtime, RuntimeError}
};

mod governor;
mod loader;
mod runtime;
mod util;
mod config;
//todo add debug print to all errors for more detail. add core:debug settings
//refactor: remove anyhow when core is stable, remove mutex blocks and make functions return result for easy error throw, wrap unsafe api in safe api and use this in the core impl instead
pub fn main() -> Result<(), MainError> {
    let gov_lifetime = GovernorLifetime::new();
    Runtime::start()?;
    ctrlc::set_handler(ctrlc_handler)?; //todo cancel stdin read when ^C is handled
    loop {
        match Runtime::park()? {
            PowerState::Shutdown => break,
            PowerState::Restart => Runtime::restart()?,
            PowerState::Cancel | PowerState::Running => {},
        }
    }
    drop(gov_lifetime); //explicit drop to ensure the GGL lifetime is exactly the lifetime of main.
    Ok(())
}

fn ctrlc_handler() {
    let gov = get_gov().expect("could not acquire governor for shutdown!");
    gov.runtime().set_power(PowerState::Shutdown);
}

pub static GOV: ArcSwapOption<Governor> = ArcSwapOption::const_empty();


#[derive(Debug, Display, Error)]
pub enum MainError {
    RuntimeError(#[from]RuntimeError),
    CtrlcError(#[from]ctrlc::Error)
}
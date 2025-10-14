use arc_swap::ArcSwapOption;

use anyhow::Result;

use crate::{
    governor::{Governor, GovernorLifetime, get_gov},
    loader::Loader,
    runtime::{PowerState, Runtime},
};

mod governor;
mod loader;
mod runtime;
mod util;

pub fn main() -> Result<()> {
    let gov_lifetime = GovernorLifetime::new()?;
    Loader::load_libraries()?;
    Runtime::init()?;
    ctrlc::set_handler(ctrlc_handler)?;
    loop {
        match Runtime::park()? {
            PowerState::Shutdown => break,
            PowerState::Restart => Runtime::restart()?,
            PowerState::Cancel | PowerState::Running => continue,
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

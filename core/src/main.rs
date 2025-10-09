use parking_lot::RwLock;

use anyhow::Result;

use crate::{governor::{write_gov, Governor, GovernorLifetime}, loader::Loader, runtime::{PowerState, Runtime}};

mod loader;
mod runtime;
mod governor;

pub fn main() -> Result<()> {
    let gov_lifetime = GovernorLifetime::new()?;
    Loader::load_libraries()?;
    Runtime::init()?;
    ctrlc::set_handler(ctrlc_handler)?;
    loop {
        match Runtime::park()? {
            Some(PowerState::Shutdown) => break,
            Some(PowerState::Restart) => Runtime::restart()?,
            Some(PowerState::Cancel) | None => continue,
        }
    }
    drop(gov_lifetime);//explicit drop to ensure the GGL lifetime is exactly the lifetime of main.
    Ok(())
}

fn ctrlc_handler() {
    let mut gov = write_gov().expect("could not acquire GGL for shutdown.");
    gov.runtime_mut().set_power(PowerState::Shutdown);
}

pub static GGL: RwLock<Option<Governor>> = RwLock::new(None);
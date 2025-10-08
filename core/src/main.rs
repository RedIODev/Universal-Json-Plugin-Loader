use parking_lot::RwLock;

use anyhow::Result;

use crate::{governor::{Governor, GovernorLifetime}, loader::Loader, runtime::{PowerState, Runtime}};

mod loader;
mod runtime;
mod governor;

pub fn main() -> Result<()> {
    let gov_lock = GovernorLifetime::new()?;
    Loader::load_libraries()?;
    Runtime::init()?;
    loop {
        match Runtime::park()? {
            Some(PowerState::Shutdown) => break,
            Some(PowerState::Restart) => Runtime::restart()?,
            Some(PowerState::Cancel) | None => continue,
        }
    }
    drop(gov_lock);
    Ok(())
}

pub static GGL: RwLock<Option<Governor>> = RwLock::new(None);
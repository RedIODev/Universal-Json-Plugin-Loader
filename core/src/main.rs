use std::sync::{LazyLock, RwLock};

use anyhow::Result;

use crate::{governor::Governor, loader::Loader, runtime::{PowerState, Runtime}};

mod loader;
mod runtime;
mod governor;

pub fn main() -> Result<()> {
    Loader::load_libraries()?;
    Runtime::init()?;
    loop {
        match Runtime::park()? {
            Some(PowerState::Shutdown) => return Ok(()),
            Some(PowerState::Restart) => Runtime::restart()?,
            Some(PowerState::Cancel) | None => continue,
        }

    }
}

pub static GGL: LazyLock<RwLock<Governor>> = LazyLock::new(|| RwLock::new(Governor::new()));
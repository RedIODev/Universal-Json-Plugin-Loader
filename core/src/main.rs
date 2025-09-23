use std::sync::{LazyLock, RwLock};

use anyhow::Result;

use crate::{governor::Governor, loader::Loader, runtime::Runtime};

mod loader;
mod runtime;
mod governor;

pub fn main() -> Result<()> {
    unsafe { Loader::load_library( "libexample.so")? };
    Runtime::init()?;

    Ok(())
}

pub static GGL: LazyLock<RwLock<Governor>> = LazyLock::new(|| RwLock::new(Governor::new()));
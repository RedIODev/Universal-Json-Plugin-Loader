use std::sync::{LazyLock, Mutex};

use anyhow::Result;

use crate::{governor::Governor, loader::Loader, runtime::Runtime};

mod loader;
mod runtime;
mod governor;

pub fn main() -> Result<()> {
    unsafe { Loader::load_library( "libexample.so")? };
    Runtime::init();

    Ok(())
}

pub static GGL: LazyLock<Mutex<Governor>> = LazyLock::new(|| Mutex::new(Governor::new()));
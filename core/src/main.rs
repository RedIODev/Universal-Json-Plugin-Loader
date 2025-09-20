use std::sync::{LazyLock, Mutex};

use anyhow::Result;

use crate::{governor::Governor, loader::Loader};

mod loader;
mod runtime;
mod governor;

pub fn main() -> Result<()> {
    unsafe { Loader::load_library( "filename")? };
    GGL.lock().unwrap().init(); 
    Ok(())
}

pub static GGL: LazyLock<Mutex<Governor>> = LazyLock::new(|| Mutex::new(Governor::new()));
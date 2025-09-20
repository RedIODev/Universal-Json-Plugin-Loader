use std::sync::{LazyLock, Mutex};

use anyhow::Result;

use crate::{loader::Loader, runtime::Runtime};

mod loader;
mod runtime;

pub fn main() -> Result<()> {
    unsafe { LOADER.lock().expect("").load_library("filename") }
}

pub static LOADER: LazyLock<Mutex<Loader>> = LazyLock::new(|| Mutex::new(Loader::new(Runtime::new())));
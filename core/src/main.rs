#![allow(clippy::missing_errors_doc)]

extern crate alloc;

use crate::
    launcher::{LaunchError, Launcher}
;

mod config;
mod governor;
mod launcher;
mod loader;
mod runtime;
mod util;
//refactor: remove mutex blocks, check dependencies before running core:init
pub fn main() -> Result<(), LaunchError> {
    Launcher::new("example-loader").launch()
}

#![allow(clippy::missing_errors_doc)]

use crate::
    launcher::{LaunchError, Launcher}
;

mod config;
mod governor;
mod launcher;
mod loader;
mod runtime;
mod util;
//refactor: remove mutex blocks
pub fn main() -> Result<(), LaunchError> {
    Launcher::new("finance-together").launch()
}

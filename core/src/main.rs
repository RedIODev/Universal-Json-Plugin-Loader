#![allow(clippy::missing_errors_doc, missing_docs, reason = "main is allowed to return any Error")]

//temporary fix for dependency version mismatch
#![allow(clippy::multiple_crate_versions, reason = "temporary warning silencing until solution is found")]

extern crate alloc;



mod config;
mod governor;
mod launcher;
mod loader;
mod runtime;
mod util;


use crate::
    launcher::{LaunchError, Launcher}
;
//refactor: remove mutex blocks, check dependencies before running core:init, pointer cast in api/misc
pub fn main() -> Result<(), LaunchError> {
    Launcher::new("example-loader").launch()
}

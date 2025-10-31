use std::env::args;

use arc_swap::ArcSwapOption;

use anyhow::Result;
use clap::Parser;

use crate::{
    config::{Cli, Config, PluginOption}, governor::{Governor, GovernorLifetime, get_gov}, loader::Loader, runtime::{PowerState, Runtime}
};

mod governor;
mod loader;
mod runtime;
mod util;
mod config;
//refactor: remove anyhow when core is stable, remove mutex blocks and make functions return result for easy error throw, wrap unsafe api in safe api and use this in the core impl instead
pub fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("{:#?}", cli);
    return Ok(());

    let gov_lifetime = GovernorLifetime::new()?;
    Runtime::start()?;
    ctrlc::set_handler(ctrlc_handler)?;
    loop {
        match Runtime::park()? {
            PowerState::Shutdown => break,
            PowerState::Restart => Runtime::restart()?,
            PowerState::Cancel | PowerState::Running => continue,
        }
    }
    drop(gov_lifetime); //explicit drop to ensure the GGL lifetime is exactly the lifetime of main.
    Ok(())
}

fn ctrlc_handler() {
    let gov = get_gov().expect("could not acquire governor for shutdown!");
    gov.runtime().set_power(PowerState::Shutdown);
}

pub static GOV: ArcSwapOption<Governor> = ArcSwapOption::const_empty();

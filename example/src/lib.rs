#![allow(clippy::print_stdout, reason = "plugins are allowed to print to std_out/err")]
#![allow(clippy::use_debug, reason = "example is allowed to print debug info")]


extern crate alloc;
use alloc::borrow::Cow;
use alloc::sync::Arc;
use core::sync::atomic::Ordering;
use core::time::Duration;
use std::{
    io::{Write as _, stdin, stdout},
    thread,
};

use arc_swap::ArcSwapOption;
use atomic_enum::atomic_enum;
use plugin_loader_api::{
    API_VERSION, ApplicationContext, ErrorMapper as _, PluginInfo, ServiceError,
    pointer_traits::{EventHandlerFunc, plugin_main, trait_fn},
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

static UUID: ArcSwapOption<Uuid> = ArcSwapOption::const_empty();
static POWER: AtomicMyPow = AtomicMyPow::new(MyPow::None);

#[atomic_enum]
#[derive(Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
enum MyPow {
    Cancel,
    #[serde(skip)]
    None, 
    Restart,
    Shutdown,
}

#[derive(Deserialize, PartialEq)]
struct PowWrap {
    command: MyPow,
    delay: Option<u32>,
    timestamp: String,
}

#[expect(clippy::single_call_fn, reason = "main function")]
#[plugin_main]
fn main(uuid: Uuid) -> PluginInfo {
    println!("Main: Test from plugin! {uuid}");
    UUID.store(Some(Arc::new(uuid)));
    PluginInfo::new::<InitTest, _, _, _>("ExamplePlugin", "0.0.2", [], API_VERSION)
}

#[trait_fn(EventHandlerFunc for PowerListener)]
fn handle<'args, F: Fn() -> Result<ApplicationContext, ServiceError>, S: Into<Cow<'args, str>>>(
    context: F,
    args: S,
) -> Result<(), ServiceError> {
    let pow_args: PowWrap = serde_json::from_str(&args.into()).error(ServiceError::InvalidString)?;
    POWER.store(pow_args.command, Ordering::Relaxed);
    if pow_args.delay.is_some() {
        let uuid = **UUID.load().as_ref().error(ServiceError::PluginInternalError)?;
        context()?.endpoint_request("core:power", uuid, json!({"command": "cancel"}).to_string())?;
        println!("canceled shutdown");
    }
    Ok(())
}

#[trait_fn(EventHandlerFunc for InitTest)]
fn handle<'args, F: Fn() -> Result<ApplicationContext, ServiceError>, S: Into<Cow<'args, str>>>(context: F, args: S) -> Result<(), ServiceError> {
    
        println!("Plugin: Init: Test from plugin! Args:{}", args.into());
        let uuid = **UUID.load().as_ref().error(ServiceError::PluginInternalError)?;
        context()?.register_event_handler::<PowerListener,_>(uuid, "core:power")?;
        println!("before while loop with {:?}", POWER.load(Ordering::Relaxed));
        while POWER.load(Ordering::Relaxed) < MyPow::Shutdown {
            let mut input = String::new();
            print!(">");
            stdout().flush().error(ServiceError::PluginInternalError)?;
            stdin().read_line(&mut input).error(ServiceError::PluginInternalError)?;
            println!("after read");
            let request_args = if input.trim() == "shutdown20000" {
                json!({"command": "shutdown", "delay": 20_000i32}).to_string()
            } else {
                json!({"command": input.trim()}).to_string()
            };
            match context()?.endpoint_request("core:power", uuid, request_args) {
                Ok(response) => println!("Response:{response}"),
                Err(err) => println!("RequestError:{err}")
            }
            thread::sleep(Duration::from_secs(1));
        }
        println!("plugin exit due to program exit or restart");
        POWER.store(MyPow::None, Ordering::Relaxed);
        Ok(())
   
}

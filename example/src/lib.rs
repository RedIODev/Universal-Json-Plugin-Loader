extern crate alloc;
use alloc::borrow::Cow;

use std::{
    io::{Write, stdin, stdout},
    sync::{Arc, atomic::Ordering},
    thread,
    time::Duration,
};

use arc_swap::ArcSwapOption;
use atomic_enum::atomic_enum;
use plugin_loader_api::{
    API_VERSION, ApplicationContext, ErrorMapper, PluginInfo, ServiceError,
    pointer_traits::{EventHandlerFunc, plugin_main, trait_fn},
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[atomic_enum]
#[derive(Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
enum MyPow {
    #[serde(skip)]
    None,
    Cancel,
    Shutdown,
    Restart,
}

#[derive(Deserialize, PartialEq)]
struct PowWrap {
    command: MyPow,
    timestamp: String,
    delay: Option<u32>,
}

#[plugin_main]
fn main(uuid: Uuid) -> PluginInfo {
    println!("Main: Test from plugin! {uuid:?}");
    UUID.store(Some(Arc::new(uuid)));
    PluginInfo::new::<InitTest, _, _, _>("ExamplePlugin", "0.0.2", [], API_VERSION)
}

static POWER: AtomicMyPow = AtomicMyPow::new(MyPow::None);
static UUID: ArcSwapOption<Uuid> = ArcSwapOption::const_empty();

#[trait_fn(EventHandlerFunc for PowerListener)]
fn handle<'a, F: Fn() -> Result<ApplicationContext, ServiceError>, S: Into<Cow<'a, str>>>(
    context: F,
    args: S,
) -> Result<(), ServiceError> {
    let args: PowWrap = serde_json::from_str(&args.into()).error(ServiceError::InvalidString)?;
    POWER.store(args.command, Ordering::Relaxed);
    if args.delay.is_some() {
        let uuid = **UUID.load().as_ref().error(ServiceError::PluginInternalError)?;
        context()?.endpoint_request("core:power", uuid, json!({"command": "cancel"}).to_string())?;
        println!("canceled shutdown");
    }
    Ok(())
}

#[trait_fn(EventHandlerFunc for InitTest)]
fn handle<'a, F: Fn() -> Result<ApplicationContext, ServiceError>, S: Into<Cow<'a, str>>>(context: F, args: S) -> Result<(), ServiceError> {
    
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
            let args = if input.trim() == "shutdown20000" {
                json!({"command": "shutdown", "delay": 20000}).to_string()
            } else {
                json!({"command": input.trim()}).to_string()
            };
            match context()?.endpoint_request("core:power", uuid, args) {
                Ok(response) => println!("Response:{response}"),
                Err(err) => println!("RequestError:{err}")
            }
            thread::sleep(Duration::from_secs(1));
        }
        println!("plugin exit due to program exit or restart");
        POWER.store(MyPow::None, Ordering::Relaxed);
        Ok(())
   
}

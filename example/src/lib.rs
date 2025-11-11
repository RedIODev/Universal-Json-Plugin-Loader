use std::{
    error::Error,
    io::{Write, stdin, stdout},
    sync::{Arc, atomic::Ordering},
    thread,
    time::Duration,
};

use arc_swap::ArcSwapOption;
use atomic_enum::atomic_enum;
use finance_together_api::{
    API_VERSION, ApplicationContext, PluginInfo,
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
    println!("Main: Test from plugin! {:?}", uuid);
    UUID.store(Some(Arc::new(uuid)));
    PluginInfo::new::<InitTest, _, _, _>("ExamplePlugin", "0.0.2", [], API_VERSION)
}

static POWER: AtomicMyPow = AtomicMyPow::new(MyPow::None);
static UUID: ArcSwapOption<Uuid> = ArcSwapOption::const_empty();

#[trait_fn(EventHandlerFunc)]
fn PowerListener<F: Fn() -> ApplicationContext, S: AsRef<str>>(context: F, args: S) {
    if let Err(e) = power_listener(context, args) {
        println!("Error: {}", e);
    }
    fn power_listener<F: Fn() -> ApplicationContext, S: AsRef<str>>(
        context: F,
        args: S,
    ) -> Result<(), Box<dyn Error>> {
        let args: PowWrap = serde_json::from_str(args.as_ref())?;
        POWER.store(args.command, Ordering::Relaxed);
        if args.delay.is_some() {
            context().endpoint_request("core:power", json!({"command": "cancel"}).to_string())?;
            println!("canceled shutdown");
        }
        Ok(())
    }
}

#[trait_fn(EventHandlerFunc)]
fn InitTest<F: Fn() -> ApplicationContext, S: AsRef<str>>(context: F, args: S) {
    if let Err(e) = init_test(context, args) {
        println!("Error: {}", e);
    }
    fn init_test<F: Fn() -> ApplicationContext, S: AsRef<str>>(
        context: F,
        args: S,
    ) -> Result<(), Box<dyn Error>> {
        println!("Plugin: Init: Test from plugin! Args:{}", args.as_ref());
        let uuid = (**UUID.load().as_ref().ok_or("Uuid None")?).clone();
        context().register_event_handler(PowerListener, uuid, "core:power")?;
        println!("before while loop with {:?}", POWER.load(Ordering::Relaxed));
        while POWER.load(Ordering::Relaxed) < MyPow::Shutdown {
            let mut input = String::new();
            print!(">");
            stdout().flush()?;
            stdin().read_line(&mut input)?;
            println!("after read");
            let args = if input.trim() == "shutdown20000" {
                json!({"command": "shutdown", "delay": 20000}).to_string()
            } else {
                json!({"command": input.trim()}).to_string()
            };
            let response = context().endpoint_request("core:power", args)?;
            println!("Response:{response}");
            thread::sleep(Duration::from_secs(1));
        }
        println!("plugin exit due to program exit or restart");
        POWER.store(MyPow::None, Ordering::Relaxed);
        Ok(())
    }
}

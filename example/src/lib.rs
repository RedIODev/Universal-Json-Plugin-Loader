use std::io::stdin;

use finance_together_api::{API_VERSION, ApplicationContext, PluginInfo, pointer_traits::{EventHandlerFunc, plugin_main, trait_fn}};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize, PartialEq)]
enum MyPow {
     Running,
    Shutdown,
    Restart,
    Cancel,
}

#[plugin_main]
fn main(uuid: Uuid) -> PluginInfo {
    println!("Main: Test from plugin! {:?}", uuid);
    PluginInfo::new::<InitTest,_,_,_>("ExamplePlugin", "0.0.2", [], API_VERSION)
}

#[trait_fn(EventHandlerFunc)]
fn InitTest<F: Fn() -> ApplicationContext, S: AsRef<str>>(context: F, args: S) {//todo: fails with Input2 while having only 2 args
    println!("Plugin: Init: Test from plugin! Args:{}", args.as_ref());
    let power = context()
        .endpoint_request("core:power", json!({"command": "state"}).to_string())
        .expect("failed to read power state!");
    let mut pow: MyPow = MyPow::Running;
    while pow != MyPow::Shutdown {
        pow = serde_json::from_str(power.response().expect("valid response")).expect("valid parse!");
        let mut input = String::new();
        stdin().read_line(&mut input).expect("read success");
        let args = if input.trim() == "shutdown4000" {
            json!({"command": "shutdown", "delay": 40000}).to_string()
        } else {
            json!({"command": input.trim()}).to_string()
        };
        let result = context().endpoint_request(
            "core:power", 
            args
        );
        match result {
            Ok(response) => println!("Response:{response}"),
            Err(err) => println!("Error:{err}")
        }
    }
} 
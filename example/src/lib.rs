use std::io::stdin;

use finance_together_api::cbindings::{emptyListString, CString, CUuid, ContextSupplier, PluginInfo};
use serde_json::json;


#[unsafe(no_mangle)]
pub unsafe extern "C" fn pluginMain(uuid: CUuid) -> PluginInfo {
    println!("Main: Test from plugin! {:?}", uuid);
    let info = PluginInfo {
        name: CString::from("ExamplePlugin"),
        version: CString::from("0.0.1"),
        dependencies: unsafe { emptyListString() },
        init_handler: Some(init_test),
    };
    info
}

unsafe extern "C" fn init_test(context: ContextSupplier, args: CString) {
    println!("Plugin: Init: Test from plugin! Args:{}", args.as_str().expect("msg"));
    let mut input = String::new();
    stdin().read_line(&mut input).expect("read success");
    let error = unsafe { context.unwrap()().endpointRequestService.unwrap()("core:power".into(), json!({"command": input.trim()}).to_string().into()) };
    println!("Plugin: error: {:?}", error)
}


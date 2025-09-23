use finance_together_api::cbindings::{createString, emptyListString, CString, CUuid, ContextSupplier, PluginInfo};


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
    println!("Init: Test from plugin! Args:{}", args.as_str().expect("msg"))
}
use finance_together_api::cbindings::{CHandlerFP, CString, CUuid, ContextSupplier};


#[unsafe(no_mangle)]
pub unsafe extern "C" fn pluginMain(uuid: CUuid) -> CHandlerFP {
    println!("Main: Test from plugin! {:?}", uuid);
    Some(init_test)
}

unsafe extern "C" fn init_test(context: ContextSupplier, args: CString) {
    println!("Init: Test from plugin! Args:{}", args.as_str().expect("msg"))
}
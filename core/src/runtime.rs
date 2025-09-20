use finance_together_api::{cbindings::{CHandler, CString, CUuid}, Handler};
use jsonschema::Validator;

use crate::LOADER;


struct Runtime {

}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn handlerRegister(handler: CHandler, plugin_id: CUuid, event_name: CString) -> bool {
    let Some(handler) = handler else {
        return false;
    };
    let Ok(mut events) = LOADER.handler.lock() else {
        return false;
    };
    let Ok(event_name) = event_name.as_str() else {
        return false;
    };
    let Some(event) = events.get_mut(event_name) else {
        return false;
    };
    event.handlers.push(StoredHandler { handler, plugin_id });
    true
}


pub(crate) struct Event {
    pub(crate) handlers: Vec<StoredHandler>,
    argument_validator: Validator,
    result_validator: Validator
}

impl Event {
    pub fn new(argument_validator: Validator, result_validator: Validator) -> Self {
        Self { handlers: Vec::new(), argument_validator, result_validator }
    }
}



pub(crate) struct StoredHandler {
    pub(crate) handler: Handler,
    pub(crate) plugin_id: CUuid
}


impl StoredHandler {
    pub fn new(handler: Handler, plugin_id: CUuid) -> Self {
        Self { handler, plugin_id }
    }
}

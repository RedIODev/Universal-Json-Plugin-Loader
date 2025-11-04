use trait_fn::trait_fn;
use uuid::Uuid;

use crate::{cbindings::{CContextSupplier, CEventHandler, CEventHandlerFP, CHandlerRegisterService, CHandlerUnregisterService, CServiceError, CString, CUuid}, safe_api::ServiceError};

// macro_rules! trait_lampda {
//     (let $l:ident for $tr:ty => $func:ident($args:tt) -> $ret:ty $body:block) => {//add args
//         struct $l;
//         impl $tr for $l {
//             fn $func($args) -> $ret $body
//         }
//     };
// }

// trait_lampda!{let XH for EventHandlerFP => handle(context: u8, args: impl AsRef<str>) -> () {

// }}

pub trait EventHandlerFP {
    fn handle(context: u8, args: impl AsRef<str>);

    fn as_fp() -> CEventHandlerFP {
        Some(event_handle_adapter::<Self>)
    }
}

unsafe extern "C" fn event_handle_adapter<E: EventHandlerFP + ?Sized>(context: CContextSupplier, args: CString) {
    E::handle(5, args.as_str().expect("Not a Valid UTF8-String!"));
}

pub trait HandlerRegisterService {
    fn event_handler_register(handler: u8, plugin_id: Uuid, event_name: impl AsRef<str>) -> u8;

    fn as_fp() -> CHandlerRegisterService {
        Some(event_handler_register_adapter::<Self>)
    }
}

unsafe extern "C" fn event_handler_register_adapter<H: HandlerRegisterService + ?Sized>(handler: CEventHandlerFP, plugin_id: CUuid, event_name: CString) -> CEventHandler {
    let Ok(event_name) = event_name.as_str() else {
        return CEventHandler::new_error(ServiceError::InvalidInput2);
    };
    H::event_handler_register(5, plugin_id.into(), event_name);
    todo!()
}

pub trait HandlerUnregisterService {
    fn event_handler_unregister(handler_id: Uuid, plugin_id: Uuid, event_name: impl AsRef<str>) -> ServiceError;

    fn as_fp() -> CHandlerUnregisterService {
        Some(event_handler_unregister_adapter::<Self>)
    }
}

unsafe extern "C" fn event_handler_unregister_adapter<H: HandlerUnregisterService + ?Sized>(handler_id: CUuid, plugin_id: CUuid, event_name: CString) -> CServiceError {
    let Ok(event_name) = event_name.as_str() else {
        return CServiceError::InvalidInput3;
    };
    H::event_handler_unregister(handler_id.into(), plugin_id.into(), event_name).into()
}

#[trait_fn(HandlerUnregisterService)]
pub fn event_handler_unregister(handler_id: Uuid, plugin_id: Uuid, event_name: impl AsRef<str>) -> ServiceError {
    ServiceError::CoreInternalError
}
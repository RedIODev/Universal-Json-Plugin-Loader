
use std::fmt::Debug;

use derive_more::Display;
use thiserror::Error;
use uuid::Uuid;

pub mod misc;
pub mod pointer_traits;

#[derive(Debug, Clone, Copy, Display, Error)]
pub enum ServiceError {
    CoreInternalError,
    InvalidInput0,
    InvalidInput1,
    InvalidInput2,
    InvalidInput3,
    InvalidInput4,
    InvalidInput5,
    InvalidInput6,
    InvalidInput7,
    NotFound,
    Unauthorized,
    Duplicate,
    PluginUninit,
    InvalidResponse,
    ShutingDown,
}

use crate::cbindings::CApplicationContext;
use crate::safe_api::pointer_traits::{EndpointRegisterServiceUnsafeFP, EndpointRequestServiceUnsafeFP, EndpointUnregisterServiceUnsafeFP, EventHandlerFuncUnsafeFP, EventRegisterServiceUnsafeFP, EventTriggerServiceUnsafeFP, EventUnregisterServiceUnsafeFP, HandlerRegisterServiceUnsafeFP, HandlerUnregisterServiceUnsafeFP};
use crate::{
    cbindings::{CEndpointResponse, CEventHandler, CServiceError, CString},
    
};

trait OkOrCoreInternalError<T> {
    fn ok_or_core(self) -> Result<T, ServiceError>;
}

impl<T> OkOrCoreInternalError<T> for Option<T> {
    fn ok_or_core(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::CoreInternalError)
    }
}

impl CServiceError {
    pub const fn to_rust(self) -> Result<(), ServiceError> {
        Err(match self {
            CServiceError::Success => return Ok(()),
            CServiceError::CoreInternalError => ServiceError::CoreInternalError,
            CServiceError::InvalidInput0 => ServiceError::InvalidInput0,
            CServiceError::InvalidInput1 => ServiceError::InvalidInput1,
            CServiceError::InvalidInput2 => ServiceError::InvalidInput2,
            CServiceError::InvalidInput3 => ServiceError::InvalidInput3,
            CServiceError::InvalidInput4 => ServiceError::InvalidInput4,
            CServiceError::InvalidInput5 => ServiceError::InvalidInput5,
            CServiceError::InvalidInput6 => ServiceError::InvalidInput6,
            CServiceError::InvalidInput7 => ServiceError::InvalidInput7,
            CServiceError::NotFound => ServiceError::NotFound,
            CServiceError::Unauthorized => ServiceError::Unauthorized,
            CServiceError::Duplicate => ServiceError::Duplicate,
            CServiceError::PluginUninit => ServiceError::PluginUninit,
            CServiceError::InvalidResponse => ServiceError::InvalidResponse,
            CServiceError::ShutingDown => ServiceError::ShutingDown,
        })
    }
}

impl ServiceError {
    pub const fn to_c(self) -> CServiceError {
        match self {
            ServiceError::CoreInternalError => CServiceError::CoreInternalError,
            ServiceError::InvalidInput0 => CServiceError::InvalidInput0,
            ServiceError::InvalidInput1 => CServiceError::InvalidInput1,
            ServiceError::InvalidInput2 => CServiceError::InvalidInput2,
            ServiceError::InvalidInput3 => CServiceError::InvalidInput3,
            ServiceError::InvalidInput4 => CServiceError::InvalidInput4,
            ServiceError::InvalidInput5 => CServiceError::InvalidInput5,
            ServiceError::InvalidInput6 => CServiceError::InvalidInput6,
            ServiceError::InvalidInput7 => CServiceError::InvalidInput7,
            ServiceError::NotFound => CServiceError::NotFound,
            ServiceError::Unauthorized => CServiceError::Unauthorized,
            ServiceError::Duplicate => CServiceError::Duplicate,
            ServiceError::PluginUninit => CServiceError::PluginUninit,
            ServiceError::InvalidResponse => CServiceError::InvalidResponse,
            ServiceError::ShutingDown => CServiceError::ShutingDown,
        }
    }
}

impl From<ServiceError> for CServiceError {
    fn from(value: ServiceError) -> Self {
        value.to_c()
    }
}

impl From<()> for CServiceError {
    fn from(_: ()) -> Self {
        Self::Success
    }
}

impl From<Result<(), ServiceError>> for CServiceError {
    fn from(value: Result<(), ServiceError>) -> Self {
        match value {
            Ok(ok) => ok.into(),
            Err(e) => e.into(),
        }
    }
}

impl From<CServiceError> for Result<(), ServiceError> {
    fn from(value: CServiceError) -> Self {
        value.to_rust()
    }
}

pub struct EventHandler {//todo add accessor function
    function: EventHandlerFuncUnsafeFP,
    handler_id: Uuid,
}

impl CEventHandler {
    pub fn to_rust(self) -> Result<EventHandler, ServiceError> {
        self.error.to_rust()?;
        let func = self.function.ok_or_core()?;

        Ok(EventHandler {
            function: func,
            handler_id: self.handler_id.into(),
        })
    }
}

impl EventHandler {
    pub fn to_c(self) -> CEventHandler {
        CEventHandler {
            function: Some(self.function),
            handler_id: self.handler_id.into(),
            error: CServiceError::Success,
        }
    }
}

impl From<EventHandler> for CEventHandler {
    fn from(value: EventHandler) -> Self {
        value.to_c()
    }
}

impl From<ServiceError> for CEventHandler {
    fn from(value: ServiceError) -> Self {
        CEventHandler::new_error(value.into())
    }
}

impl From<Result<EventHandler, ServiceError>> for CEventHandler {
    fn from(value: Result<EventHandler, ServiceError>) -> Self {
        match value {
            Ok(ok) => ok.into(),
            Err(e) => e.into()
        }
    }
}

impl From<CEventHandler> for Result<EventHandler, ServiceError> {
    fn from(value: CEventHandler) -> Self {
        value.to_rust()
    }
}

pub struct EndpointResponse { //todo add accessor function
    response: CString
}

impl CEndpointResponse {
    pub fn to_rust(self) -> Result<EndpointResponse, ServiceError> {
        self.error.to_rust()?;
        Ok(EndpointResponse { response: self.response })
    }
}

impl EndpointResponse {
    pub fn to_c(self) -> CEndpointResponse {
        CEndpointResponse { response: self.response, error: CServiceError::Success }
    }
}

impl From<EndpointResponse> for CEndpointResponse {
    fn from(value: EndpointResponse) -> Self {
        value.to_c()
    }
}

impl From<ServiceError> for CEndpointResponse {
    fn from(value: ServiceError) -> Self {
        CEndpointResponse::new_error(value.into())
    }
}

impl From<Result<EndpointResponse, ServiceError>> for CEndpointResponse {
    fn from(value: Result<EndpointResponse, ServiceError>) -> Self {
        match value {
            Ok(ok) => ok.into(),
            Err(e) => e.into()
        }
    }
}

impl From<CEndpointResponse> for Result<EndpointResponse, ServiceError> {
    fn from(value: CEndpointResponse) -> Self {
        value.to_rust()
    }
}

pub struct ApplicationContext { // todo add accessor functions
    handler_register_service: HandlerRegisterServiceUnsafeFP,
    handler_unregister_service: HandlerUnregisterServiceUnsafeFP,
    event_register_service: EventRegisterServiceUnsafeFP,
    event_unregister_service: EventUnregisterServiceUnsafeFP,
    event_trigger_service: EventTriggerServiceUnsafeFP,
    endpoint_register_service: EndpointRegisterServiceUnsafeFP,
    endpoint_unregister_service: EndpointUnregisterServiceUnsafeFP,
    endpoint_request_service: EndpointRequestServiceUnsafeFP
}

impl CApplicationContext {
    pub fn to_rust(self) -> Result<ApplicationContext, ServiceError> {
        Ok(ApplicationContext { 
            handler_register_service: self.handlerRegisterService.ok_or_core()?, 
            handler_unregister_service: self.handlerUnregisterService.ok_or_core()?, 
            event_register_service: self.eventRegisterService.ok_or_core()?, 
            event_unregister_service: self.eventUnregisterService.ok_or_core()?, 
            event_trigger_service: self.eventTriggerService.ok_or_core()?, 
            endpoint_register_service: self.endpointRegisterService.ok_or_core()?, 
            endpoint_unregister_service: self.endpointUnregisterService.ok_or_core()?, 
            endpoint_request_service: self.endpointRequestService.ok_or_core()? 
        })

    }
}

impl ApplicationContext {
    pub fn to_c(self) -> CApplicationContext {
        CApplicationContext { 
            handlerRegisterService: Some(self.handler_register_service), 
            handlerUnregisterService: Some(self.handler_unregister_service), 
            eventRegisterService: Some(self.event_register_service), 
            eventUnregisterService: Some(self.event_unregister_service), 
            eventTriggerService: Some(self.event_trigger_service), 
            endpointRegisterService: Some(self.endpoint_register_service), 
            endpointUnregisterService: Some(self.endpoint_unregister_service), 
            endpointRequestService: Some(self.endpoint_request_service) 
        }
    }
}

impl From<ApplicationContext> for CApplicationContext {
    fn from(value: ApplicationContext) -> Self {
        value.to_c()
    }
}

// pub trait EventHandlerFP: Fn() -> ApplicationContext {}

// impl<T> EventHandlerFP for T where T: Fn() -> ApplicationContext {}

// #[derive(Clone)]
// pub struct EventHandler<F> { //Find a better way around the fp and trait impl problem.
//     pub function: F,
//     pub handler_id: Uuid,
// }

// #[derive(Debug, Clone, Copy, Display, Error)]
// pub struct NullFunctionPointerError;

// impl<F> TryFrom<CEventHandler> for EventHandler<F> where F:  {
//     type Error = NullFunctionPointerError;

//     fn try_from(value: CEventHandler) -> Result<Self, Self::Error> {
//         let Some(function) = value.function else {
//             return Err(NullFunctionPointerError);
//         };
//         Ok(EventHandler {
//             function: Rc::new(|| unsafe { function().try_into().expect("msg")}),
//             handler_id: value.handler_id.into(),
//         })
//     }
// }

// impl From<EventHandler> for CEventHandler {
//     fn from(value: EventHandler) -> Self {
//         Self {
//             function: Some(value.function),
//             handler_id: value.handler_id.into(),
//             error: CServiceError::Success,
//         }
//     }
// }

// impl<STR:Into<Box<str>>> From<Result<STR, ServiceError>> for EndpointResponse {
//     fn from(value: Result<STR, ServiceError>) -> Self {
//         match value {
//             Ok(str) => EndpointResponse { response: str.into().into(), error: CServiceError::Success },
//             Err(e) => EndpointResponse { response: CString::from(""), error: e.into() }
//         }
//     }
// }

// #[derive(Debug, Display, Error, EnumFrom)]
// pub enum EndpointResponseError {
//     ServiceError(ServiceError),
//     StringConventError(StringConventError)
// }

// impl TryFrom<EndpointResponse> for String {
//     type Error = EndpointResponseError;

//     fn try_from(value: EndpointResponse) -> Result<Self, Self::Error> {
//         let _:() = value.error.try_into()?;
//         return Ok(value.response.as_str()?.to_owned());
//     }
// }

// impl TryFrom<EndpointResponse> for Box<str> {
//     type Error = EndpointResponseError;

//     fn try_from(value: EndpointResponse) -> Result<Self, Self::Error> {
//         let _:() = value.error.try_into()?;
//         return Ok(value.response.as_str()?.into());
//     }
// }

// #[derive(Clone, Copy, Debug, Display, Error)]
// pub struct InvalidFunctionPointerError;

// pub struct ApplicationContext {

// }

// impl From<ApplicationContext> for CApplicationContext {
//     fn from(value: ApplicationContext) -> Self {
//         todo!()
//     }
// }

// impl TryFrom<CApplicationContext> for ApplicationContext {
//     type Error = InvalidFunctionPointerError;

//     fn try_from(value: CApplicationContext) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

// mod fp_traits {
//     use crate::cbindings::{ApplicationContext as CApplicationContext, CString};
//     use crate::safe_api::{ApplicationContext};
//     use crate::cbindings::ContextSupplier as CContextSupplier;

//     pub trait ContextSupplierTrait {
//         fn get_context() -> ApplicationContext;
//     }

//     unsafe extern "C" fn context_supplier_wrapper<C: ContextSupplierTrait>() -> CApplicationContext {
//         C::get_context().into()
//     }

//     pub trait ContextSupplierFP: Fn() -> ApplicationContext {}

//     impl<T> ContextSupplierFP for T where T: Fn() -> ApplicationContext {}

//     pub trait EventHandlerTrait {
//         fn handle(context: impl ContextSupplierFP, args: impl Into<Box<str>>);
//     }

//     unsafe extern "C" fn event_handler_fp_wrapper<E: EventHandlerTrait>(context_supplier: CContextSupplier, args: CString) {
//         let supplier = || unsafe { context_supplier.expect("msg")().try_into().expect("msg")};
//         E::handle(supplier, args.as_str().expect("Invalid Argument String"));
//     }

//     pub trait HandlerRegisterServiceTrait {

//     }
// }

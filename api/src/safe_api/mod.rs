use std::fmt::Debug;
use std::rc::Rc;
use std::str::FromStr;

use derive_enum_from_into::EnumFrom;
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

use crate::{cbindings::{CEventHandler, CString, CEndpointResponse, CServiceError}, safe_api::misc::StringConventError};

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

impl TryFrom<CServiceError> for () {
    type Error = ServiceError;

    fn try_from(value: CServiceError) -> Result<Self, Self::Error> {
        value.to_rust()
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
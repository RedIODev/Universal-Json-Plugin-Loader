use std::marker::PhantomData;

use trait_fn::fn_trait;
use uuid::Uuid;

use crate::{
    cbindings::{
        CContextSupplier, CEndpointResponse, CEventHandler, CEventHandlerFP, CServiceError,
        CString, CUuid,
    },
    safe_api::ServiceError,
};

#[fn_trait]
pub trait EventHandlerFunc {
    fn safe<S: AsRef<str>>(context: u8, args: S);

    unsafe extern "C" fn adapter(context: CContextSupplier, args: CString) {
        Self::safe(5, args.as_str().expect("Not a Valid UTF8-String!"));
    }

    fn from_fp<S: AsRef<str>>(self: EventHandlerFuncUnsafeFP) -> impl Fn(u8, S) {
        move |context, args| unsafe { self(todo!(), args.as_ref().into()) }
    }
}

#[fn_trait]
pub trait HandlerRegisterService {
    fn safe<S: AsRef<str>>(handler: u8, plugin_id: Uuid, event_name: S) -> u8;

    unsafe extern "C" fn adapter(
        handler: CEventHandlerFP,
        plugin_id: CUuid,
        event_name: CString,
    ) -> CEventHandler {
        let Ok(event_name) = event_name.as_str() else {
            return CEventHandler::new_error(CServiceError::InvalidInput2);
        };
        Self::safe(todo!(), plugin_id.into(), event_name);
        todo!()
    }

    fn from_fp<S: AsRef<str>>(self: HandlerRegisterServiceUnsafeFP) -> impl Fn(u8, Uuid, S) -> u8 {
        move |context, plugin_id, event_name| unsafe {
            self(None, plugin_id.into(), event_name.as_ref().into());
            todo!()
        }
    }
}

#[fn_trait]
pub trait HandlerUnregisterService {
    fn safe<S: AsRef<str>>(
        handler_id: Uuid,
        plugin_id: Uuid,
        event_name: S,
    ) -> Result<(), ServiceError>;

    unsafe extern "C" fn adapter(
        handler_id: CUuid,
        plugin_id: CUuid,
        event_name: CString,
    ) -> CServiceError {
        let Ok(event_name) = event_name.as_str() else {
            return CServiceError::InvalidInput2;
        };
        Self::safe(handler_id.into(), plugin_id.into(), event_name).into()
    }
    fn from_fp<S: AsRef<str>>(
        self: HandlerUnregisterServiceUnsafeFP,
    ) -> impl Fn(Uuid, Uuid, S) -> Result<(), ServiceError> {
        move |handler_id, plugin_id, event_name| unsafe {
            self(
                handler_id.into(),
                plugin_id.into(),
                event_name.as_ref().into(),
            )
            .into()
        }
    }
}

#[fn_trait]
pub trait EventRegisterService {
    fn safe<S: AsRef<str>, T: AsRef<str>>(
        event_schema: S,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<(), ServiceError>;

    unsafe extern "C" fn adapter(
        event_schema: CString,
        plugin_id: CUuid,
        event_name: CString,
    ) -> CServiceError {
        let Ok(event_schema) = event_schema.as_str() else {
            return CServiceError::InvalidInput0;
        };
        let Ok(event_name) = event_name.as_str() else {
            return CServiceError::InvalidInput2;
        };
        Self::safe(event_schema, plugin_id.into(), event_name).into()
    }

    fn from_fp<S: AsRef<str>, T: AsRef<str>>(
        self: EventRegisterServiceUnsafeFP,
    ) -> impl Fn(S, Uuid, T) -> Result<(), ServiceError> {
        move |event_schema, plugin_id, event_name| unsafe {
            self(
                event_schema.as_ref().into(),
                plugin_id.into(),
                event_name.as_ref().into(),
            )
            .into()
        }
    }
}

#[fn_trait]
pub trait EventUnregisterService {
    fn safe<S: AsRef<str>>(plugin_id: Uuid, event_name: S) -> Result<(), ServiceError>;

    unsafe extern "C" fn adapter(plugin_id: CUuid, event_name: CString) -> CServiceError {
        let Ok(event_name) = event_name.as_str() else {
            return CServiceError::InvalidInput1;
        };
        Self::safe(plugin_id.into(), event_name).into()
    }

    fn from_fp<S: AsRef<str>>(
        self: EventUnregisterServiceUnsafeFP,
    ) -> impl Fn(Uuid, S) -> Result<(), ServiceError> {
        move |plugin_id, event_name| unsafe {
            self(plugin_id.into(), event_name.as_ref().into()).into()
        }
    }
}

#[fn_trait]
pub trait EventTriggerService {
    fn safe<S: AsRef<str>, T: AsRef<str>>(
        plugin_id: Uuid,
        event_name: S,
        args: T,
    ) -> Result<(), ServiceError>;

    unsafe extern "C" fn adapter(
        plugin_id: CUuid,
        event_name: CString,
        args: CString,
    ) -> CServiceError {
        let Ok(event_name) = event_name.as_str() else {
            return CServiceError::InvalidInput1;
        };
        let Ok(args) = args.as_str() else {
            return CServiceError::InvalidInput2;
        };
        Self::safe(plugin_id.into(), event_name, args).into()
    }

    fn from_fp<S: AsRef<str>, T: AsRef<str>>(
        self: EventTriggerServiceUnsafeFP,
    ) -> impl Fn(Uuid, S, T) -> Result<(), ServiceError> {
        move |plugin_id, event_name, args| unsafe {
            self(
                plugin_id.into(),
                event_name.as_ref().into(),
                args.as_ref().into(),
            )
            .into()
        }
    }
}

#[fn_trait]
pub trait RequestHandlerFunc {
    fn safe<S: AsRef<str>>(context_supplier: u8, args: S) -> u8;

    unsafe extern "C" fn adapter(
        context_supplier: CContextSupplier,
        args: CString,
    ) -> CEndpointResponse {
        let Ok(args) = args.as_str() else {
            return CEndpointResponse::new_error(CServiceError::InvalidInput1);
        };
        Self::safe(todo!(), args);
        todo!()
    }

    fn from_fp<S: AsRef<str>>(self: RequestHandlerFuncUnsafeFP) -> impl Fn(u8, S) -> u8 {
        move |context_supplier, args| unsafe {
            self(todo!(), args.as_ref().into());
            todo!()
        }
    }
}

#[fn_trait]
pub trait EndpointRegisterService {
    fn safe<S: AsRef<str>, T: AsRef<str>, Q: AsRef<str>, R: AsRef<str>, F: Fn(u8, R) -> u8>(
        args_schema: S,
        response_schema: T,
        plugin_id: Uuid,
        endpoint_name: Q,
        _t: PhantomData<R>,
        handler: F,
    ) -> Result<(), ServiceError>;

    unsafe extern "C" fn adapter(
        args_schema: CString,
        response_schema: CString,
        plugin_id: CUuid,
        endpoint_name: CString,
        handler: Option<RequestHandlerFuncUnsafeFP>,
    ) -> CServiceError {
        let Ok(args_schema) = args_schema.as_str() else {
            return CServiceError::InvalidInput0;
        };
        let Ok(response_schema) = response_schema.as_str() else {
            return CServiceError::InvalidInput1;
        };
        let Ok(endpoint_name) = endpoint_name.as_str() else {
            return CServiceError::InvalidInput3;
        };
        let Some(handler) = handler else {
            return CServiceError::InvalidInput4;
        };
        Self::safe::<&str, &str, &str, &str, _>(
            args_schema,
            response_schema,
            plugin_id.into(),
            endpoint_name,
            PhantomData::<&str>,
            handler.to_safe(),
        )
        .into()
    }

    fn from_fp<
        S: AsRef<str>,
        T: AsRef<str>,
        Q: AsRef<str>,
        R: AsRef<str>,
        F: RequestHandlerFunc,
    >(
        self: EndpointRegisterServiceUnsafeFP,
    ) -> impl Fn(S, T, Uuid, Q, PhantomData<R>, F) -> Result<(), ServiceError> {
        move |args_schema, response_schema, plugin_id, endpoint_name, _, _| unsafe {
            self(
                args_schema.as_ref().into(),
                response_schema.as_ref().into(),
                plugin_id.into(),
                endpoint_name.as_ref().into(),
                Some(<F as RequestHandlerFunc>::unsafe_fp()),
            )
            .into()
        }
    }
}

#[fn_trait]
pub trait EndpointUnregisterService {
    fn safe<S: AsRef<str>>(plugin_id: Uuid, endpoint_name: S) -> Result<(), ServiceError>;

    unsafe extern "C" fn adapter(plugin_id: CUuid, endpoint_name: CString) -> CServiceError {
        let Ok(endpoint_name) = endpoint_name.as_str() else {
            return CServiceError::InvalidInput1
        };
        Self::safe(plugin_id.into(), endpoint_name).into()
    }

    fn from_fp<S: AsRef<str>>(self: EndpointUnregisterServiceUnsafeFP) -> impl Fn(Uuid, S) -> Result<(), ServiceError> {
        move |plugin_id, endpoint_name| unsafe {
            self(plugin_id.into(), endpoint_name.as_ref().into()).into()
        }
    }
}

#[fn_trait]
pub trait EndpointRequestService {
    fn safe<S: AsRef<str>, T: AsRef<str>>(endpoint_name: S, args: T) -> u8;

    unsafe extern "C" fn adapter(endpoint_name: CString, args: CString) -> CEndpointResponse {
        let Ok(endpoint_name) = endpoint_name.as_str() else {
            return CEndpointResponse::new_error(CServiceError::InvalidInput0);
        };
        let Ok(args) = args.as_str() else {
            return CEndpointResponse::new_error(CServiceError::InvalidInput1);
        };
        Self::safe(endpoint_name, args);
        todo!()
    }

    fn from_fp<S: AsRef<str>, T: AsRef<str>>(self: EndpointRequestServiceUnsafeFP) -> impl Fn(S, T) -> u8 {
        move |endpoint_name, args| unsafe {
            self(endpoint_name.as_ref().into(), args.as_ref().into());
            todo!()
        }
    }
}

// #[trait_fn(HandlerUnregisterService)]
// pub fn HandlerUnregisterServiceImpl<S: AsRef<str>>(
//     handler_id: Uuid,
//     plugin_id: Uuid,
//     event_name: S,
// ) -> ServiceError {
//     ServiceError::CoreInternalError
// }

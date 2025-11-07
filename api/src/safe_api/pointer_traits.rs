use std::marker::PhantomData;

use trait_fn::fn_trait;
use uuid::Uuid;

use crate::{
    cbindings::{
        CApplicationContext, CContextSupplier, CEndpointResponse, CEventHandler, CEventHandlerFP, CServiceError, CString, CUuid
    },
    safe_api::{ApplicationContext, EndpointResponse, EventHandler, ServiceError},
};

#[fn_trait]
pub trait ContextSupplier {
    fn safe() -> ApplicationContext;

    unsafe extern "C" fn adapter() -> CApplicationContext {
        Self::safe().to_c()
    }

    fn from_fp(self: ContextSupplierUnsafeFP) -> impl Fn() -> Result<ApplicationContext, ServiceError> {
        move || unsafe {self().to_rust()}
    }
}

#[fn_trait]
pub trait EventHandlerFunc {
    fn safe<F: Fn() -> ApplicationContext, S: AsRef<str>>(context: F, args: S);

    unsafe extern "C" fn adapter(context: CContextSupplier, args: CString) {
        let context = context.expect("Null function pointers are invalid!").to_safe();
        let context = || context().expect("ApplicationContext must only contain valid fp!");
        Self::safe(context, args.as_str().expect("Not a Valid UTF8-String!"));
    }

    fn from_fp<C: ContextSupplier, S: AsRef<str>>(self: EventHandlerFuncUnsafeFP) -> impl Fn(C, S) {
        move |_, args| unsafe { self(Some(C::unsafe_fp()), args.as_ref().into()) }
    }
}

#[fn_trait]
pub trait HandlerRegisterService {
    fn safe<C: ContextSupplier, S: AsRef<str>, H: Fn(C, S), T: AsRef<str>>(handler: H, plugin_id: Uuid, event_name: T, _: PhantomData<(C, S)>) -> Result<EventHandler, ServiceError>;

    unsafe extern "C" fn adapter<C: ContextSupplier>(
        handler: CEventHandlerFP,
        plugin_id: CUuid,
        event_name: CString,
    ) -> CEventHandler {
        let Some(handler) = handler else {
            return CEventHandler::new_error(CServiceError::InvalidInput0);
        };
        let Ok(event_name) = event_name.as_str() else {
            return CEventHandler::new_error(CServiceError::InvalidInput2);
        };
        Self::safe(handler.to_safe::<C, &str>(), plugin_id.into(), event_name, PhantomData).into()
    }

    fn from_fp<E: EventHandlerFunc, T: AsRef<str>>(self: HandlerRegisterServiceUnsafeFP) -> impl Fn(E, Uuid, T) -> Result<EventHandler, ServiceError> {
        move |_, plugin_id, event_name| unsafe {
            self(Some(E::unsafe_fp()), plugin_id.into(), event_name.as_ref().into()).into()
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
    fn safe<F: Fn() -> ApplicationContext, S: AsRef<str>>(context_supplier: F, args: S) -> Result<EndpointResponse, ServiceError>;

    unsafe extern "C" fn adapter(
        context_supplier: CContextSupplier,
        args: CString,
    ) -> CEndpointResponse {
        let Ok(args) = args.as_str() else {
            return CEndpointResponse::new_error(CServiceError::InvalidInput1);
        };
        let Some(context) = context_supplier else {
            return CEndpointResponse::new_error(CServiceError::InvalidInput0);
        };
        let context = || context.to_safe()().expect("ApplicationContext must only contain valid fp!");
        Self::safe(context, args).into()
    }

    fn from_fp<C: ContextSupplier, S: AsRef<str>>(self: RequestHandlerFuncUnsafeFP) -> impl Fn(C, S) -> Result<EndpointResponse, ServiceError> {
        move |_, args| unsafe {
            self(Some(C::unsafe_fp()), args.as_ref().into()).into()
        }
    }
}

#[fn_trait]
pub trait EndpointRegisterService {
    fn safe<C:ContextSupplier, S: AsRef<str>, T: AsRef<str>, Q: AsRef<str>, R: AsRef<str>, F: Fn(C, R) -> Result<EndpointResponse, ServiceError>>(
        args_schema: S,
        response_schema: T,
        plugin_id: Uuid,
        endpoint_name: Q,
        _t: PhantomData<(C, R)>,
        handler: F,
    ) -> Result<(), ServiceError>;

    unsafe extern "C" fn adapter<C: ContextSupplier>(
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
        Self::safe(
            args_schema,
            response_schema,
            plugin_id.into(),
            endpoint_name,
            PhantomData::<(C, &str)>,
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
                Some(F::unsafe_fp()),
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
    fn safe<S: AsRef<str>, T: AsRef<str>>(endpoint_name: S, args: T) -> Result<EndpointResponse, ServiceError>;

    unsafe extern "C" fn adapter(endpoint_name: CString, args: CString) -> CEndpointResponse {
        let Ok(endpoint_name) = endpoint_name.as_str() else {
            return CEndpointResponse::new_error(CServiceError::InvalidInput0);
        };
        let Ok(args) = args.as_str() else {
            return CEndpointResponse::new_error(CServiceError::InvalidInput1);
        };
        Self::safe(endpoint_name, args).into()
    }

    fn from_fp<S: AsRef<str>, T: AsRef<str>>(self: EndpointRequestServiceUnsafeFP) -> impl Fn(S, T) -> Result<EndpointResponse, ServiceError> {
        move |endpoint_name, args| unsafe {
            self(endpoint_name.as_ref().into(), args.as_ref().into()).into()
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

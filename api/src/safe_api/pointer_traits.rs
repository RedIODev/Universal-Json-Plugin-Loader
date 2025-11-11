
use uuid::Uuid;

use crate::{
    ErrorMapper, cbindings::{
        CApplicationContext, CContextSupplier, CEndpointResponse, CEventHandler, CEventHandlerFP, CServiceError, CString, CUuid
    }, safe_api::{ApplicationContext, EndpointResponse, EventHandler, ServiceError}
};

pub use trait_fn::*;

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
    fn safe<T: AsRef<str>>(handler: EventHandlerFuncUnsafeFP, plugin_id: Uuid, event_name: T) -> Result<EventHandler, ServiceError>;

    unsafe extern "C" fn adapter<C: ContextSupplier>(
        handler: CEventHandlerFP,
        plugin_id: CUuid,
        event_name: CString,
    ) -> CEventHandler {
        let handler = match handler.err_null_fp() {
            Ok(handler) => handler,
            Err(e) => return e.into()
        };
        let event_name = match event_name.as_str().err_invalid_str() {
            Ok(event_name) => event_name,
            Err(e) => return e.into()
        };
        Self::safe(handler, plugin_id.into(), event_name).into()
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
        let event_name = match event_name.as_str().err_invalid_str() {
            Ok(event_name) => event_name,
            Err(e) => return e.into()
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
        let event_schema = match event_schema.as_str().err_invalid_str() {
            Ok(event_schema) => event_schema,
            Err(e) => return e.into()
        };
        let event_name = match event_name.as_str().err_invalid_str() {
            Ok(event_name) => event_name,
            Err(e) => return e.into()
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
        let event_name = match event_name.as_str().err_invalid_str() {
            Ok(event_name) => event_name,
            Err(e) => return e.into()
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
        let event_name = match event_name.as_str().err_invalid_str() {
            Ok(event_name) => event_name,
            Err(e) => return e.into()
        };
        let args = match args.as_str().err_invalid_str() {
            Ok(args) => args,
            Err(e) => return e.into()
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
        let args = match args.as_str().err_invalid_str() {
            Ok(args) => args,
            Err(e) => return e.into()
        };
        let context = match context_supplier.err_null_fp() {
            Ok(context) => context,
            Err(e) => return e.into()
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
    fn safe<S: AsRef<str>, T: AsRef<str>, Q: AsRef<str>>(
        args_schema: S,
        response_schema: T,
        plugin_id: Uuid,
        endpoint_name: Q,
        handler: RequestHandlerFuncUnsafeFP,
    ) -> Result<(), ServiceError>;

    unsafe extern "C" fn adapter(
        args_schema: CString,
        response_schema: CString,
        plugin_id: CUuid,
        endpoint_name: CString,
        handler: Option<RequestHandlerFuncUnsafeFP>,
    ) -> CServiceError {
        let args_schema = match args_schema.as_str().err_invalid_str() {
            Ok(args_schema) => args_schema,
            Err(e) => return e.into()
        };
        let response_schema = match response_schema.as_str().err_invalid_str() {
            Ok(response_schema) => response_schema,
            Err(e) => return e.into()
        };
        let endpoint_name = match endpoint_name.as_str().err_invalid_str() {
            Ok(endpoint_name) => endpoint_name,
            Err(e) => return e.into()
        };
        let handler = match handler.err_null_fp() {
            Ok(handler) => handler,
            Err(e) => return e.into()
        };
        Self::safe(
            args_schema,
            response_schema,
            plugin_id.into(),
            endpoint_name,
            handler,
        )
        .into()
    }

    fn from_fp<
        S: AsRef<str>,
        T: AsRef<str>,
        Q: AsRef<str>,
        F: RequestHandlerFunc,
    >(
        self: EndpointRegisterServiceUnsafeFP,
    ) -> impl Fn(S, T, Uuid, Q) -> Result<(), ServiceError> {
        move |args_schema, response_schema, plugin_id, endpoint_name| unsafe {
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
        let endpoint_name = match endpoint_name.as_str().err_invalid_str() {
            Ok(endpoint_name) => endpoint_name,
            Err(e) => return e.into()
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
        let endpoint_name = match endpoint_name.as_str().err_invalid_str() {
            Ok(endpoint_name) => endpoint_name,
            Err(e) => return e.into()
        };
        let args = match args.as_str().err_invalid_str() {
            Ok(args) => args,
            Err(e) => return e.into()
        };

        Self::safe(endpoint_name, args).into()
    }

    fn from_fp<S: AsRef<str>, T: AsRef<str>>(self: EndpointRequestServiceUnsafeFP) -> impl Fn(S, T) -> Result<EndpointResponse, ServiceError> {
        move |endpoint_name, args| unsafe {
            self(endpoint_name.as_ref().into(), args.as_ref().into()).into()
        }
    }
}
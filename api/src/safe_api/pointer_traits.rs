#![allow(clippy::must_use_candidate)]

use std::borrow::Cow;

use uuid::Uuid;

use crate::{
    ErrorMapper,
    cbindings::{
        CApplicationContext, CContextSupplier, CEventHandler, CEventHandlerFP,
        CServiceError, CString, CUuid,
    },
    safe_api::{ApplicationContext, EventHandler, ServiceError},
};

pub use trait_fn::*;

#[fn_trait]
pub trait ContextSupplier {
    #[sig]
    fn supply() -> ApplicationContext;

    ///
    ///
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`ContextSupplier::supply`].
    ///
    #[adapter]
    unsafe extern "C" fn adapter() -> CApplicationContext {
        Self::supply().to_c()
    }

    #[fp_adapter]
    fn to_safe_fp(
        self: ContextSupplierUnsafeFP,
    ) -> impl Fn() -> Result<ApplicationContext, ServiceError> {
        move || unsafe { self().to_rust() }
    }
}

#[fn_trait]
pub trait EventHandlerFunc {
    #[sig]
    fn handle<'a, F: Fn() -> ApplicationContext, S: Into<Cow<'a, str>>>(context: F, args: S) -> Result<(), ServiceError>;

    ///
    ///
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventHandlerFunc::handle`].
    ///
    #[adapter]
    unsafe extern "C" fn adapter(context: CContextSupplier, args: CString) {
        fn adapter_inner<T:EventHandlerFunc + ?Sized>(context: CContextSupplier, args: &CString) -> Result<(), ServiceError> {
            let context = context.error(ServiceError::NullFunctionPointer)?
                .to_safe_fp();
            let context = || context().error(ServiceError::CoreInternalError).expect("Invalid Application Context");
            T::handle(context, args.as_str().error(ServiceError::InvalidString)?)
        }

        adapter_inner::<Self>(context, &args).expect("Handler encountered an error!");
    }

    #[fp_adapter]
    fn to_safe_fp<C: ContextSupplier, S: Into<CString>>(
        self: EventHandlerFuncUnsafeFP,
    ) -> impl Fn(C, S) {
        move |_, args| unsafe { self(Some(C::adapter_fp()), args.into()) }
    }
}

#[fn_trait]
pub trait EventHandlerRegisterService {
    #[sig]
    fn register<T: AsRef<str>>(
        handler: EventHandlerFuncUnsafeFP,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<EventHandler, ServiceError>;

    ///
    ///
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventHandlerRegisterService::register`].
    ///
    #[adapter]
    unsafe extern "C" fn adapter(
        handler: CEventHandlerFP,
        plugin_id: CUuid,
        event_name: CString,
    ) -> CEventHandler {
        let handler = match handler.error(ServiceError::NullFunctionPointer) {
            Ok(handler) => handler,
            Err(e) => return e.into(),
        };
        let event_name = match event_name.as_str().error(ServiceError::InvalidString) {
            Ok(event_name) => event_name,
            Err(e) => return e.into(),
        };
        Self::register(handler, plugin_id.into(), event_name).into()
    }

    #[fp_adapter]
    fn to_safe_fp<E: EventHandlerFunc, T: Into<CString>>(
        self: EventHandlerRegisterServiceUnsafeFP,
    ) -> impl Fn(E, Uuid, T) -> Result<EventHandler, ServiceError> {
        move |_, plugin_id, event_name| unsafe {
            self(Some(E::adapter_fp()), plugin_id.into(), event_name.into()).into()
        }
    }
}

#[fn_trait]
pub trait EventHandlerUnregisterService {
    #[sig]
    fn unregister<S: AsRef<str>>(
        handler_id: Uuid,
        plugin_id: Uuid,
        event_name: S,
    ) -> Result<(), ServiceError>;

    ///
    ///
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventHandlerUnregisterService::unregister`].
    ///
    #[adapter]
    unsafe extern "C" fn adapter(
        handler_id: CUuid,
        plugin_id: CUuid,
        event_name: CString,
    ) -> CServiceError {
        let event_name = match event_name.as_str().error(ServiceError::InvalidString) {
            Ok(event_name) => event_name,
            Err(e) => return e.into(),
        };
        Self::unregister(handler_id.into(), plugin_id.into(), event_name).into()
    }

    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>>(
        self: EventHandlerUnregisterServiceUnsafeFP,
    ) -> impl Fn(Uuid, Uuid, S) -> Result<(), ServiceError> {
        move |handler_id, plugin_id, event_name| unsafe {
            self(handler_id.into(), plugin_id.into(), event_name.into()).into()
        }
    }
}

#[fn_trait]
pub trait EventRegisterService {
    #[sig]
    fn register<S: AsRef<str>, T: AsRef<str>>(
        event_schema: S,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<(), ServiceError>;

    ///
    ///
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventRegisterService::register`].
    ///
    #[adapter]
    unsafe extern "C" fn adapter(
        event_schema: CString,
        plugin_id: CUuid,
        event_name: CString,
    ) -> CServiceError {
        let event_schema = match event_schema.as_str().error(ServiceError::InvalidString) {
            Ok(event_schema) => event_schema,
            Err(e) => return e.into(),
        };
        let event_name = match event_name.as_str().error(ServiceError::InvalidString) {
            Ok(event_name) => event_name,
            Err(e) => return e.into(),
        };
        Self::register(event_schema, plugin_id.into(), event_name).into()
    }

    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>, T: Into<CString>>(
        self: EventRegisterServiceUnsafeFP,
    ) -> impl Fn(S, Uuid, T) -> Result<(), ServiceError> {
        move |event_schema, plugin_id, event_name| unsafe {
            self(event_schema.into(), plugin_id.into(), event_name.into()).into()
        }
    }
}

#[fn_trait]
pub trait EventUnregisterService {
    #[sig]
    fn unregister<S: AsRef<str>>(plugin_id: Uuid, event_name: S) -> Result<(), ServiceError>;

    ///
    ///
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventUnregisterService::unregister`].
    ///
    #[adapter]
    unsafe extern "C" fn adapter(plugin_id: CUuid, event_name: CString) -> CServiceError {
        let event_name = match event_name.as_str().error(ServiceError::InvalidString) {
            Ok(event_name) => event_name,
            Err(e) => return e.into(),
        };
        Self::unregister(plugin_id.into(), event_name).into()
    }

    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>>(
        self: EventUnregisterServiceUnsafeFP,
    ) -> impl Fn(Uuid, S) -> Result<(), ServiceError> {
        move |plugin_id, event_name| unsafe { self(plugin_id.into(), event_name.into()).into() }
    }
}

#[fn_trait]
pub trait EventTriggerService {
    #[sig]
    fn trigger<S: AsRef<str>, T: AsRef<str>>(
        plugin_id: Uuid,
        event_name: S,
        args: T,
    ) -> Result<(), ServiceError>;

    ///
    ///
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventTriggerService::trigger`].
    ///
    #[adapter]
    unsafe extern "C" fn adapter(
        plugin_id: CUuid,
        event_name: CString,
        args: CString,
    ) -> CServiceError {
        let event_name = match event_name.as_str().error(ServiceError::InvalidString) {
            Ok(event_name) => event_name,
            Err(e) => return e.into(),
        };
        let args = match args.as_str().error(ServiceError::InvalidString) {
            Ok(args) => args,
            Err(e) => return e.into(),
        };
        Self::trigger(plugin_id.into(), event_name, args).into()
    }

    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>, T: Into<CString>>(
        self: EventTriggerServiceUnsafeFP,
    ) -> impl Fn(Uuid, S, T) -> Result<(), ServiceError> {
        move |plugin_id, event_name, args| unsafe {
            self(plugin_id.into(), event_name.into(), args.into()).into()
        }
    }
}

#[fn_trait]
pub trait RequestHandlerFunc {
    #[sig]
    fn handle<'a, F: Fn() -> ApplicationContext, S: Into<Cow<'a, str>>, T: AsRef<str>>(
        context_supplier: F,
        plugin_name: T,
        args: S,
    ) -> Result<String, ServiceError>;

    ///
    ///
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`RequestHandlerFunc::handle`].
    ///
    #[adapter]
    unsafe extern "C" fn adapter(
        context_supplier: CContextSupplier,
        plugin_name: CString,
        args: CString,
    ) -> CString {
        let args = match args.as_str().error(ServiceError::InvalidString) {
            Ok(args) => args,
            Err(e) => return e.into(),
        };
        let plugin_name = match plugin_name.as_str().error(ServiceError::InvalidString) {
            Ok(plugin_name) => plugin_name,
            Err(e) => return e.into(),
        };
        let context = match context_supplier.error(ServiceError::NullFunctionPointer) {
            Ok(context) => context,
            Err(e) => return e.into(),
        };
        let context =
            || context.to_safe_fp()().expect("ApplicationContext must only contain valid fp!");
        Self::handle(context, plugin_name, args).into()
    }

    #[fp_adapter]
    fn to_safe_fp<'a, C: ContextSupplier, S: Into<CString>, T: Into<CString>>(
        self: RequestHandlerFuncUnsafeFP,
    ) -> impl Fn(C, S, T) -> Result<String, ServiceError> {
        move |_, plugin_name,args| unsafe { self(Some(C::adapter_fp()), plugin_name.into(), args.into()).into() }
    }
}

#[fn_trait]
pub trait EndpointRegisterService {
    #[sig]
    fn register<S: AsRef<str>, T: AsRef<str>, Q: AsRef<str>>(
        args_schema: S,
        response_schema: T,
        plugin_id: Uuid,
        endpoint_name: Q,
        handler: RequestHandlerFuncUnsafeFP,
    ) -> Result<(), ServiceError>;

    ///
    ///
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EndpointRegisterService::register`].
    ///
    #[adapter]
    unsafe extern "C" fn adapter(
        args_schema: CString,
        response_schema: CString,
        plugin_id: CUuid,
        endpoint_name: CString,
        handler: Option<RequestHandlerFuncUnsafeFP>,
    ) -> CServiceError {
        let args_schema = match args_schema.as_str().error(ServiceError::InvalidString) {
            Ok(args_schema) => args_schema,
            Err(e) => return e.into(),
        };
        let response_schema = match response_schema.as_str().error(ServiceError::InvalidString) {
            Ok(response_schema) => response_schema,
            Err(e) => return e.into(),
        };
        let endpoint_name = match endpoint_name.as_str().error(ServiceError::InvalidString) {
            Ok(endpoint_name) => endpoint_name,
            Err(e) => return e.into(),
        };
        let handler = match handler.error(ServiceError::NullFunctionPointer) {
            Ok(handler) => handler,
            Err(e) => return e.into(),
        };
        Self::register(
            args_schema,
            response_schema,
            plugin_id.into(),
            endpoint_name,
            handler,
        )
        .into()
    }

    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>, T: Into<CString>, Q: Into<CString>, F: RequestHandlerFunc>(
        self: EndpointRegisterServiceUnsafeFP,
    ) -> impl Fn(S, T, Uuid, Q) -> Result<(), ServiceError> {
        move |args_schema, response_schema, plugin_id, endpoint_name| unsafe {
            self(
                args_schema.into(),
                response_schema.into(),
                plugin_id.into(),
                endpoint_name.into(),
                Some(F::adapter_fp()),
            )
            .into()
        }
    }
}

#[fn_trait]
pub trait EndpointUnregisterService {
    #[sig]
    fn unregister<S: AsRef<str>>(plugin_id: Uuid, endpoint_name: S) -> Result<(), ServiceError>;

    ///
    ///
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EndpointUnregisterService::unregister`].
    ///
    #[adapter]
    unsafe extern "C" fn adapter(plugin_id: CUuid, endpoint_name: CString) -> CServiceError {
        let endpoint_name = match endpoint_name.as_str().error(ServiceError::InvalidString) {
            Ok(endpoint_name) => endpoint_name,
            Err(e) => return e.into(),
        };
        Self::unregister(plugin_id.into(), endpoint_name).into()
    }

    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>>(
        self: EndpointUnregisterServiceUnsafeFP,
    ) -> impl Fn(Uuid, S) -> Result<(), ServiceError> {
        move |plugin_id, endpoint_name| unsafe {
            self(plugin_id.into(), endpoint_name.into()).into()
        }
    }
}

#[fn_trait]
pub trait EndpointRequestService {
    #[sig]
    fn request<'a, S: AsRef<str>, T: Into<Cow<'a, str>>>(
        endpoint_name: S,
        plugin_id: Uuid,
        args: T,
    ) -> Result<String, ServiceError>;

    ///
    ///
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EndpointRequestService::request`].
    ///
    #[adapter]
    unsafe extern "C" fn adapter(endpoint_name: CString, plugin_id: CUuid, args: CString) -> CString {
        let endpoint_name = match endpoint_name.as_str().error(ServiceError::InvalidString) {
            Ok(endpoint_name) => endpoint_name,
            Err(e) => return e.into(),
        };
        let args = match args.as_str().error(ServiceError::InvalidString) {
            Ok(args) => args,
            Err(e) => return e.into(),
        };

        Self::request(endpoint_name, plugin_id.into(), args).into()
    }

    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>, T: Into<CString>>(
        self: EndpointRequestServiceUnsafeFP,
    ) -> impl Fn(S, Uuid, T) -> Result<String, ServiceError> {
        move |endpoint_name, plugin_id,args| unsafe { self(endpoint_name.into(), plugin_id.into(), args.into()).into() }
    }
}

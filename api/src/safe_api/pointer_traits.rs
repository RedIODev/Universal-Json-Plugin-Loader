#![allow(clippy::must_use_candidate, reason = "many false positives in this module")]
#![allow(clippy::undocumented_unsafe_blocks, reason = "all undocumented unsafe blocks in this module are calling defined C-Apis")]


extern crate alloc;

use alloc::borrow::Cow;
use alloc::string::String;

use uuid::Uuid;

use crate::{
    ErrorMapper as _, cbindings::{
        CApplicationContext, CContextSupplier, CEventHandler, CEventHandlerFP,
        CServiceError, CString, CUuid,
    }, misc::ToCString as _, safe_api::{ApplicationContext, EventHandler, ServiceError}
};

pub use proc_macros::*;

///
/// `ContextSupplier` `fn_trait`.
/// 
/// # Function Traits
/// Function Traits creates a bridge between unsafe C function pointers, implementations of this trait and safe function pointers.
/// 
#[fn_trait]
pub trait ContextSupplier {
    ///
    /// Supplies the `ApplicationContext` to interact with the plugin system for C code. 
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`ContextSupplier::supply`].
    ///
    #[adapter]
    #[inline]
    unsafe extern "C" fn c_supplier() -> CApplicationContext {
        Self::supply().to_c()
    }

    ///
    /// Supplies the `ApplicationContext` to interact with the plugin system.
    /// 
    #[sig]
    fn supply() -> ApplicationContext;


    #[fp_adapter]
    #[inline]
    fn to_safe_fp(
        self: ContextSupplierUnsafeFP,
    ) -> impl Fn() -> Result<ApplicationContext, ServiceError> {
        move || unsafe { self().to_rust() }
    }
}

///
/// `EventHandler` `fn_trait`.
/// 
/// # Function Traits
/// Function Traits creates a bridge between unsafe C function pointers, implementations of this trait and safe function pointers.
/// 
#[fn_trait]
pub trait EventHandlerFunc {
    ///
    /// Handles the `EventHandler` triggers from C code.
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventHandlerFunc::handle`].
    ///
    #[adapter]
    #[inline]
    unsafe extern "C" fn c_handle(c_context_supplier: CContextSupplier, c_args: CString) -> CServiceError {
        let args = match c_args.as_str().error(ServiceError::InvalidString) {
            Ok(args) => args,
            Err(error) => return error.into()
        };

        let context_supplier = match c_context_supplier.error(ServiceError::NullFunctionPointer) {
            Ok(context_supplier) => context_supplier,
            Err(error) => return error.into()
        };

        Self::handle(context_supplier.to_safe_fp(), args).into()
    }
    
    ///
    /// Handles the `EventHandler` callback.
    /// # Errors
    /// Calling an `EventHandler` callback may fail when it doesn't follow the c-api correctly or has other errors.
    /// 
    #[sig]
    fn handle<'args, F: Fn() -> Result<ApplicationContext, ServiceError>, S: Into<Cow<'args, str>>>(context: F, args: S) -> Result<(), ServiceError>;


    #[fp_adapter]
    fn to_safe_fp<C: ContextSupplier, S: Into<CString>>(
        self: EventHandlerFuncUnsafeFP,
    ) -> impl Fn(C, S) -> Result<(), ServiceError> {
        move |_, args| unsafe { self(Some(C::c_supplier_fp()), args.into()).into() }
    }
}

///
/// `EventHandlerRegisterService` `fn_trait`.
/// 
/// # Function Traits
/// Function Traits creates a bridge between unsafe C function pointers, implementations of this trait and safe function pointers.
/// 
#[fn_trait]
pub trait EventHandlerRegisterService {
    ///
    /// Registers a new `EventHandler` to an event from C code.
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventHandlerRegisterService::register`].
    ///
    #[adapter]
    #[inline]
    unsafe extern "C" fn c_register(
        c_handler: CEventHandlerFP,
        plugin_id: CUuid,
        c_event_name: CString,
    ) -> CEventHandler {
        let handler = match c_handler.error(ServiceError::NullFunctionPointer) {
            Ok(handler) => handler,
            Err(error) => return error.into(),
        };
        let event_name = match c_event_name.as_str().error(ServiceError::InvalidString) {
            Ok(event_name) => event_name,
            Err(error) => return error.into(),
        };
        Self::register(handler, plugin_id.into(), event_name).into()
    }

    ///
    /// Registers a new `EventHandler` to an event.
    /// # Errors
    /// One reason the registration might fail is that the `handler_id` was already
    /// registered in which case the old value stays unchanged and the new registration fails.
    /// 
    #[sig]
    fn register<T: AsRef<str>>(
        handler: EventHandlerFuncUnsafeFP,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<EventHandler, ServiceError>;


    #[fp_adapter]
    fn to_safe_fp<E: EventHandlerFunc, T: Into<CString>>(
        self: EventHandlerRegisterServiceUnsafeFP,
    ) -> impl Fn(Uuid, T) -> Result<EventHandler, ServiceError> {
        move | plugin_id, event_name| unsafe {
            self(Some(E::c_handle_fp()), plugin_id.into(), event_name.into()).into()
        }
    }
}

///
/// `EventHandlerUnregisterService` `fn_trait`.
/// 
/// # Function Traits
/// Function Traits creates a bridge between unsafe C function pointers, implementations of this trait and safe function pointers.
/// 
#[fn_trait]
pub trait EventHandlerUnregisterService {
    ///
    /// Unregisters an `EventHandler` from an event from C code.
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventHandlerUnregisterService::unregister`].
    ///
    #[adapter]
    #[inline]
    unsafe extern "C" fn c_unregister(
        handler_id: CUuid,
        plugin_id: CUuid,
        c_event_name: CString,
    ) -> CServiceError {
        let event_name = match c_event_name.as_str().error(ServiceError::InvalidString) {
            Ok(event_name) => event_name,
            Err(error) => return error.into(),
        };
        Self::unregister(handler_id.into(), plugin_id.into(), event_name).into()
    }

    ///
    /// Unregisters an `EventHandler` from an event.
    /// # Errors
    /// The unregistration might fail because no handler with the id can be found
    /// or the given `plugin_id` wasn't used when registering the handler.
    /// 
    #[sig]
    fn unregister<S: AsRef<str>>(
        handler_id: Uuid,
        plugin_id: Uuid,
        event_name: S,
    ) -> Result<(), ServiceError>;


    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>>(
        self: EventHandlerUnregisterServiceUnsafeFP,
    ) -> impl Fn(Uuid, Uuid, S) -> Result<(), ServiceError> {
        move |handler_id, plugin_id, event_name| unsafe {
            self(handler_id.into(), plugin_id.into(), event_name.into()).into()
        }
    }
}

///
/// `EventRegisterService` `fn_trait`.
/// 
/// # Function Traits
/// Function Traits creates a bridge between unsafe C function pointers, implementations of this trait and safe function pointers.
/// 
#[fn_trait]
pub trait EventRegisterService {
    ///
    /// Registers a new `Event` from C code.
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventRegisterService::register`].
    ///
    #[adapter]
    #[inline]
    unsafe extern "C" fn c_register(
        c_event_schema: CString,
        plugin_id: CUuid,
        c_event_name: CString,
    ) -> CServiceError {
        let event_schema = match c_event_schema.as_str().error(ServiceError::InvalidString) {
            Ok(event_schema) => event_schema,
            Err(error) => return error.into(),
        };
        let event_name = match c_event_name.as_str().error(ServiceError::InvalidString) {
            Ok(event_name) => event_name,
            Err(error) => return error.into(),
        };
        Self::register(event_schema, plugin_id.into(), event_name).into()
    }

    ///
    /// Registers a new `Event`.
    /// # Errors
    /// The registration might fail, when the schema is invalid, the name contains a ':', the plugin is not found,
    /// or the event name was already registered for this plugin.
    /// 
    #[sig]
    fn register<S: AsRef<str>, T: AsRef<str>>(
        event_schema: S,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<(), ServiceError>;


    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>, T: Into<CString>>(
        self: EventRegisterServiceUnsafeFP,
    ) -> impl Fn(S, Uuid, T) -> Result<(), ServiceError> {
        move |event_schema, plugin_id, event_name| unsafe {
            self(event_schema.into(), plugin_id.into(), event_name.into()).into()
        }
    }
}

///
/// `EventUnregisterService` `fn_trait`.
/// 
/// # Function Traits
/// Function Traits creates a bridge between unsafe C function pointers, implementations of this trait and safe function pointers.
/// 
#[fn_trait]
pub trait EventUnregisterService {
    ///
    /// Unregisters an `Event` from C code.
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventUnregisterService::unregister`].
    ///
    #[adapter]
    #[inline]
    unsafe extern "C" fn c_unregister(plugin_id: CUuid, c_event_name: CString) -> CServiceError {
        let event_name = match c_event_name.as_str().error(ServiceError::InvalidString) {
            Ok(event_name) => event_name,
            Err(error) => return error.into(),
        };
        Self::unregister(plugin_id.into(), event_name).into()
    }

    ///
    /// Unregisters an `Event`.
    /// # Errors
    /// The unregistration might fail because no event with such name was found
    /// or the given `plugin_id` wasn't used when registering the event.
    /// 
    #[sig]
    fn unregister<S: AsRef<str>>(plugin_id: Uuid, event_name: S) -> Result<(), ServiceError>;

    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>>(
        self: EventUnregisterServiceUnsafeFP,
    ) -> impl Fn(Uuid, S) -> Result<(), ServiceError> {
        move |plugin_id, event_name| unsafe { self(plugin_id.into(), event_name.into()).into() }
    }
}

///
/// `EventTriggerService` `fn_trait`.
/// 
/// # Function Traits
/// Function Traits creates a bridge between unsafe C function pointers, implementations of this trait and safe function pointers.
/// 
#[fn_trait]
pub trait EventTriggerService {
    ///
    /// Triggers an `Event` from C code.
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EventTriggerService::trigger`].
    ///
    #[adapter]
    #[inline]
    unsafe extern "C" fn c_trigger(
        plugin_id: CUuid,
        c_event_name: CString,
        c_args: CString,
    ) -> CServiceError {
        let event_name = match c_event_name.as_str().error(ServiceError::InvalidString) {
            Ok(event_name) => event_name,
            Err(error) => return error.into(),
        };
        let args = match c_args.as_str().error(ServiceError::InvalidString) {
            Ok(args) => args,
            Err(error) => return error.into(),
        };
        Self::trigger(plugin_id.into(), event_name, args).into()
    }

    ///
    /// Triggers an `Event`.
    /// # Errors
    /// The trigger of an event might fail, when the arguments aren't valid according to the events schema,
    /// the `event_name` could not be found, or the `plugin_id` is not the owner of the event.
    /// 
    #[sig]
    fn trigger<S: AsRef<str>, T: AsRef<str>>(
        plugin_id: Uuid,
        event_name: S,
        args: T,
    ) -> Result<(), ServiceError>;


    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>, T: Into<CString>>(
        self: EventTriggerServiceUnsafeFP,
    ) -> impl Fn(Uuid, S, T) -> Result<(), ServiceError> {
        move |plugin_id, event_name, args| unsafe {
            self(plugin_id.into(), event_name.into(), args.into()).into()
        }
    }
}

///
/// `RequestHandlerFunc` `fn_trait`.
/// 
/// # Function Traits
/// Function Traits creates a bridge between unsafe C function pointers, implementations of this trait and safe function pointers.
/// 
#[fn_trait]
pub trait RequestHandlerFunc {
    ///
    /// Handles the request from C code.
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`RequestHandlerFunc::handle`].
    ///
    #[adapter]
    #[inline]
    unsafe extern "C" fn c_handle(
        c_context_supplier: CContextSupplier,
        c_plugin_name: CString,
        c_args: CString,
    ) -> CString {
        let args = match c_args.as_str().error(ServiceError::InvalidString) {
            Ok(args) => args,
            Err(error) => return error.into(),
        };
        let plugin_name = match c_plugin_name.as_str().error(ServiceError::InvalidString) {
            Ok(plugin_name) => plugin_name,
            Err(error) => return error.into(),
        };
        let context_supplier = match c_context_supplier.error(ServiceError::NullFunctionPointer) {
            Ok(context) => context,
            Err(error) => return error.into(),
        };
        Self::handle(context_supplier.to_safe_fp(), plugin_name, args).to_c_string()
    }

    ///
    /// Handles the request.
    /// # Errors
    /// Calling an `RequestHandler` callback may fail when it doesn't follow the c-api correctly or has other errors.
    /// 
    #[sig]
    fn handle<'args, F: Fn() -> Result<ApplicationContext, ServiceError>, S: Into<Cow<'args, str>>, T: AsRef<str>>(
        context_supplier: F,
        plugin_name: T,
        args: S,
    ) -> Result<String, ServiceError>;


    #[fp_adapter]
    fn to_safe_fp<C: ContextSupplier, S: Into<CString>, T: Into<CString>>(
        self: RequestHandlerFuncUnsafeFP,
    ) -> impl Fn(C, S, T) -> Result<String, ServiceError> {
        move |_, plugin_name,args| unsafe { self(Some(C::c_supplier_fp()), plugin_name.into(), args.into()).into() }
    }
}

///
/// `EndpointRegisterService` `fn_trait`.
/// 
/// # Function Traits
/// Function Traits creates a bridge between unsafe C function pointers, implementations of this trait and safe function pointers.
/// 
#[fn_trait]
pub trait EndpointRegisterService {
    ///
    /// Registers a new `Endpoint` from C code.
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EndpointRegisterService::register`].
    ///
    #[adapter]
    #[inline]
    unsafe extern "C" fn c_register(
        c_args_schema: CString,
        c_response_schema: CString,
        c_plugin_id: CUuid,
        c_endpoint_name: CString,
        c_handler: Option<RequestHandlerFuncUnsafeFP>,
    ) -> CServiceError {
        let args_schema = match c_args_schema.as_str().error(ServiceError::InvalidString) {
            Ok(args_schema) => args_schema,
            Err(error) => return error.into(),
        };
        let response_schema = match c_response_schema.as_str().error(ServiceError::InvalidString) {
            Ok(response_schema) => response_schema,
            Err(error) => return error.into(),
        };
        let endpoint_name = match c_endpoint_name.as_str().error(ServiceError::InvalidString) {
            Ok(endpoint_name) => endpoint_name,
            Err(error) => return error.into(),
        };
        let handler = match c_handler.error(ServiceError::NullFunctionPointer) {
            Ok(handler) => handler,
            Err(error) => return error.into(),
        };
        Self::register(
            args_schema,
            response_schema,
            c_plugin_id.into(),
            endpoint_name,
            handler,
        )
        .into()
    }

    ///
    /// Registers a new `Endpoint`.
    /// # Errors
    /// The registration of an endpoint might fail because the `endpoint_name` contained a ':' character, the schema aren't valid,
    /// the plugin was not found, or the endpoint name was already registered for the plugin.
    /// 
    #[sig]
    fn register<S: AsRef<str>, T: AsRef<str>, Q: AsRef<str>>(
        args_schema: S,
        response_schema: T,
        plugin_id: Uuid,
        endpoint_name: Q,
        handler: RequestHandlerFuncUnsafeFP,
    ) -> Result<(), ServiceError>;


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
                Some(F::c_handle_fp()),
            )
            .into()
        }
    }
}

///
/// `EndpointUnregisterService` `fn_trait`.
/// 
/// # Function Traits
/// Function Traits creates a bridge between unsafe C function pointers, implementations of this trait and safe function pointers.
/// 
#[fn_trait]
pub trait EndpointUnregisterService {
    ///
    /// Unregisters an `Endpoint` from C code.
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EndpointUnregisterService::unregister`].
    ///
    #[adapter]
    #[inline]
    unsafe extern "C" fn c_unregister(plugin_id: CUuid, c_endpoint_name: CString) -> CServiceError {
        let endpoint_name = match c_endpoint_name.as_str().error(ServiceError::InvalidString) {
            Ok(endpoint_name) => endpoint_name,
            Err(error) => return error.into(),
        };
        Self::unregister(plugin_id.into(), endpoint_name).into()
    }

    ///
    /// Unregisters an `Endpoint`.
    /// # Errors
    /// The unregistration might fail because no endpoint with such name was found
    /// or the given `plugin_id` wasn't used when registering the endpoint.
    /// 
    #[sig]
    fn unregister<S: AsRef<str>>(plugin_id: Uuid, endpoint_name: S) -> Result<(), ServiceError>;


    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>>(
        self: EndpointUnregisterServiceUnsafeFP,
    ) -> impl Fn(Uuid, S) -> Result<(), ServiceError> {
        move |plugin_id, endpoint_name| unsafe {
            self(plugin_id.into(), endpoint_name.into()).into()
        }
    }
}

///
/// `EndpointRequestService` `fn_trait`.
/// 
/// # Function Traits
/// Function Traits creates a bridge between unsafe C function pointers, implementations of this trait and safe function pointers.
/// 
#[fn_trait]
pub trait EndpointRequestService {
    ///
    /// Makes a request from C code.
    /// # Safety
    /// This adapter method is designed to be called from C code.
    /// It is safe to call with valid arguments and does the same as [`EndpointRequestService::request`].
    ///
    #[adapter]
    #[inline]
    unsafe extern "C" fn c_request(c_endpoint_name: CString, c_plugin_id: CUuid, c_args: CString) -> CString {
        let endpoint_name = match c_endpoint_name.as_str().error(ServiceError::InvalidString) {
            Ok(endpoint_name) => endpoint_name,
            Err(error) => return error.into(),
        };
        let args = match c_args.as_str().error(ServiceError::InvalidString) {
            Ok(args) => args,
            Err(error) => return error.into(),
        };

        Self::request(endpoint_name, c_plugin_id.into(), args).to_c_string()
    }

    ///
    /// Makes a request.
    /// # Errors
    /// The request might fail, when the arguments aren't valid for the according to the endpoints schema,
    /// the endpoint name could not be found or the response from the handler isn't valid according to the endpoints response schema.
    /// 
    #[sig]
    fn request<'args, S: AsRef<str>, T: Into<Cow<'args, str>>>(
        endpoint_name: S,
        plugin_id: Uuid,
        args: T,
    ) -> Result<String, ServiceError>;


    #[fp_adapter]
    fn to_safe_fp<S: Into<CString>, T: Into<CString>>(
        self: EndpointRequestServiceUnsafeFP,
    ) -> impl Fn(S, Uuid, T) -> Result<String, ServiceError> {
        move |endpoint_name, plugin_id,args| unsafe { self(endpoint_name.into(), plugin_id.into(), args.into()).into() }
    }
}

use std::fmt::Debug;

use std::fmt::Display;
use std::hash::Hash;

use derive_more::Display;
use thiserror::Error;
use uuid::Uuid;

pub mod pointer_traits;


use crate::cbindings::CApiVersion;
use crate::cbindings::{CApplicationContext, CList_String, CPluginInfo};
use crate::cbindings::{CEndpointResponse, CEventHandler, CServiceError, CString};
use crate::misc::ApiMiscError;
use crate::safe_api::pointer_traits::{
    ContextSupplier, EndpointRegisterService, EndpointRegisterServiceFPAdapter,
    EndpointRegisterServiceUnsafeFP, EndpointRequestService, EndpointRequestServiceFPAdapter,
    EndpointRequestServiceUnsafeFP, EndpointUnregisterService, EndpointUnregisterServiceUnsafeFP,
    EventHandlerFunc, EventHandlerFuncFPAdapter, EventHandlerFuncUnsafeFP,
    EventHandlerRegisterService, EventHandlerRegisterServiceFPAdapter,
    EventHandlerRegisterServiceUnsafeFP, EventHandlerUnregisterService,
    EventHandlerUnregisterServiceFPAdapter, EventHandlerUnregisterServiceUnsafeFP,
    EventRegisterService, EventRegisterServiceFPAdapter, EventRegisterServiceUnsafeFP,
    EventTriggerService, EventTriggerServiceFPAdapter, EventTriggerServiceUnsafeFP,
    EventUnregisterService, EventUnregisterServiceFPAdapter, EventUnregisterServiceUnsafeFP,
    RequestHandlerFunc,
};

pub trait ErrorMapper<T> {
    fn err_core(self) -> Result<T, ServiceError>;
    fn err_null_fp(self) -> Result<T, ServiceError>;
    fn err_invalid_str(self) -> Result<T, ServiceError>;
    fn err_invalid_json(self) -> Result<T, ServiceError>;
    fn err_invalid_schema(self) -> Result<T, ServiceError>;
    fn err_invalid_api(self) -> Result<T, ServiceError>;
    fn err_not_found(self) -> Result<T, ServiceError>;
    fn err_unauthorized(self) -> Result<T, ServiceError>;
    fn err_duplicate(self) -> Result<T, ServiceError>;
    fn err_plugin_uninit(self) -> Result<T, ServiceError>;
    fn err_shutting_down(self) -> Result<T, ServiceError>;
}

impl<T> ErrorMapper<T> for Option<T> {
    fn err_core(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::CoreInternalError)
    }

    fn err_null_fp(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::NullFunctionPointer)
    }

    fn err_invalid_str(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::InvalidString)
    }

    fn err_invalid_json(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::InvalidJson)
    }

    fn err_invalid_schema(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::InvalidSchema)
    }

    fn err_invalid_api(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::InvalidApi)
    }

    fn err_not_found(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::NotFound)
    }

    fn err_unauthorized(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::Unauthorized)
    }

    fn err_duplicate(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::Duplicate)
    }

    fn err_plugin_uninit(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::PluginUninit)
    }

    fn err_shutting_down(self) -> Result<T, ServiceError> {
        self.ok_or(ServiceError::ShutingDown)
    }
}

impl<T, E> ErrorMapper<T> for Result<T, E> {
    fn err_core(self) -> Result<T, ServiceError> {
        self.ok().err_core()
    }

    fn err_null_fp(self) -> Result<T, ServiceError> {
        self.ok().err_null_fp()
    }

    fn err_invalid_str(self) -> Result<T, ServiceError> {
        self.ok().err_invalid_str()
    }

    fn err_invalid_json(self) -> Result<T, ServiceError> {
        self.ok().err_invalid_json()
    }

    fn err_invalid_schema(self) -> Result<T, ServiceError> {
        self.ok().err_invalid_schema()
    }

    fn err_invalid_api(self) -> Result<T, ServiceError> {
        self.ok().err_invalid_api()
    }

    fn err_not_found(self) -> Result<T, ServiceError> {
        self.ok().err_not_found()
    }

    fn err_unauthorized(self) -> Result<T, ServiceError> {
        self.ok().err_unauthorized()
    }

    fn err_duplicate(self) -> Result<T, ServiceError> {
        self.ok().err_duplicate()
    }

    fn err_plugin_uninit(self) -> Result<T, ServiceError> {
        self.ok().err_plugin_uninit()
    }

    fn err_shutting_down(self) -> Result<T, ServiceError> {
        self.ok().err_shutting_down()
    }
}
// pub trait OkOrCoreInternalError<T> {
//     fn ok_or_service(self, ) -> Result<T, ServiceError>;
// }

// impl<T> OkOrCoreInternalError<T> for Option<T> {
//     fn ok_or_service(self) -> Result<T, ServiceError> {
//         self.ok_or(ServiceError::CoreInternalError)
//     }
// }

// impl<T, E> OkOrCoreInternalError<T> for Result<T, E> {
//     fn ok_or_service(self) -> Result<T, ServiceError> {
//         self.map_err(|_| ServiceError::CoreInternalError)
//     }
// }

impl CServiceError {
    pub const fn to_rust(self) -> Result<(), ServiceError> {
        Err(match self {
            CServiceError::Success => return Ok(()),
            CServiceError::CoreInternalError => ServiceError::CoreInternalError,
            CServiceError::NullFunctionPointer => ServiceError::NullFunctionPointer,
            CServiceError::InvalidString => ServiceError::InvalidString,
            CServiceError::InvalidJson => ServiceError::InvalidJson,
            CServiceError::InvalidSchema => ServiceError::InvalidSchema,
            CServiceError::InvalidApi => ServiceError::InvalidApi,
            CServiceError::NotFound => ServiceError::NotFound,
            CServiceError::Unauthorized => ServiceError::Unauthorized,
            CServiceError::Duplicate => ServiceError::Duplicate,
            CServiceError::PluginUninit => ServiceError::PluginUninit,
            CServiceError::ShutingDown => ServiceError::ShutingDown,
        })
    }
}

impl ServiceError {
    pub const fn to_c(self) -> CServiceError {
        match self {
            ServiceError::CoreInternalError => CServiceError::CoreInternalError,
            ServiceError::NullFunctionPointer => CServiceError::NullFunctionPointer,
            ServiceError::InvalidString => CServiceError::InvalidString,
            ServiceError::InvalidJson => CServiceError::InvalidJson,
            ServiceError::InvalidSchema => CServiceError::InvalidSchema,
            ServiceError::InvalidApi => CServiceError::InvalidApi,
            ServiceError::NotFound => CServiceError::NotFound,
            ServiceError::Unauthorized => CServiceError::Unauthorized,
            ServiceError::Duplicate => CServiceError::Duplicate,
            ServiceError::PluginUninit => CServiceError::PluginUninit,
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

#[derive(Clone, Copy, Debug, Eq)]
pub struct EventHandler {
    function: EventHandlerFuncUnsafeFP,
    handler_id: Uuid,
}


impl CEventHandler {
    pub fn to_rust(self) -> Result<EventHandler, ServiceError> {
        self.error.to_rust()?;
        let func = self.function.err_null_fp()?;

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

    pub fn new_unsafe(function: EventHandlerFuncUnsafeFP, handler_id: Uuid) -> Self {
        Self {
            function,
            handler_id,
        }
    }

    pub fn new<E: EventHandlerFunc>(handler_id: Uuid) -> Self {
        Self {
            function: E::adapter_fp(),
            handler_id,
        }
    }

    pub fn handle<C: ContextSupplier, S: Into<CString>>(&self, context_supplier: C, args: S) {
        self.function.to_safe_fp()(context_supplier, args)
    }

    pub fn handler(&self) -> EventHandlerFuncUnsafeFP {
        self.function
    }

    pub fn id(&self) -> Uuid {
        self.handler_id
    }
}

impl PartialEq for EventHandler {
    fn eq(&self, other: &Self) -> bool {
        self.handler_id == other.handler_id
    }
}

impl Hash for EventHandler {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.handler_id.hash(state);
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
            Err(e) => e.into(),
        }
    }
}

impl From<CEventHandler> for Result<EventHandler, ServiceError> {
    fn from(value: CEventHandler) -> Self {
        value.to_rust()
    }
}

pub struct EndpointResponse {
    response: CString,
}

impl CEndpointResponse {
    pub fn to_rust(self) -> Result<EndpointResponse, ServiceError> {
        self.error.to_rust()?;
        Ok(EndpointResponse {
            response: self.response,
        })
    }
}

impl EndpointResponse {
    pub fn to_c(self) -> CEndpointResponse {
        CEndpointResponse {
            response: self.response,
            error: CServiceError::Success,
        }
    }

    pub fn new<S: Into<CString>>(response: S) -> Self {
        Self {
            response: response.into(),
        }
    }

    pub fn response(&self) -> Result<&str, ApiMiscError> {
        self.response.as_str()
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
            Err(e) => e.into(),
        }
    }
}

impl From<CEndpointResponse> for Result<EndpointResponse, ServiceError> {
    fn from(value: CEndpointResponse) -> Self {
        value.to_rust()
    }
}

impl Display for EndpointResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.response.as_str().expect("invalid string printed!"), f)
    }
}

pub struct ApplicationContext {
    handler_register_service: EventHandlerRegisterServiceUnsafeFP,
    handler_unregister_service: EventHandlerUnregisterServiceUnsafeFP,
    event_register_service: EventRegisterServiceUnsafeFP,
    event_unregister_service: EventUnregisterServiceUnsafeFP,
    event_trigger_service: EventTriggerServiceUnsafeFP,
    endpoint_register_service: EndpointRegisterServiceUnsafeFP,
    endpoint_unregister_service: EndpointUnregisterServiceUnsafeFP,
    endpoint_request_service: EndpointRequestServiceUnsafeFP,
}

impl CApplicationContext {
    pub fn to_rust(self) -> Result<ApplicationContext, ServiceError> {
        Ok(ApplicationContext {
            handler_register_service: self.handlerRegisterService.err_null_fp()?,
            handler_unregister_service: self.handlerUnregisterService.err_null_fp()?,
            event_register_service: self.eventRegisterService.err_null_fp()?,
            event_unregister_service: self.eventUnregisterService.err_null_fp()?,
            event_trigger_service: self.eventTriggerService.err_null_fp()?,
            endpoint_register_service: self.endpointRegisterService.err_null_fp()?,
            endpoint_unregister_service: self.endpointUnregisterService.err_null_fp()?,
            endpoint_request_service: self.endpointRequestService.err_null_fp()?,
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
            endpointRequestService: Some(self.endpoint_request_service),
        }
    }

    pub fn register_event_handler<E: EventHandlerFunc, T: Into<CString>>(
        &self,
        handler: E,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<EventHandler, ServiceError> {
        self.handler_register_service.to_safe_fp()(handler, plugin_id, event_name)
    }

    pub fn unregister_event_handler<S: Into<CString>>(
        &self,
        handler_id: Uuid,
        plugin_id: Uuid,
        event_name: S,
    ) -> Result<(), ServiceError> {
        self.handler_unregister_service.to_safe_fp()(handler_id, plugin_id, event_name)
    }

    pub fn register_event<S: Into<CString>, T: Into<CString>>(
        &self,
        args_schema: S,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<(), ServiceError> {
        self.event_register_service.to_safe_fp()(args_schema, plugin_id, event_name)
    }

    pub fn unregister_event<S: Into<CString>>(
        &self,
        plugin_id: Uuid,
        event_name: S,
    ) -> Result<(), ServiceError> {
        self.event_unregister_service.to_safe_fp()(plugin_id, event_name)
    }

    pub fn trigger_event<S: Into<CString>, T: Into<CString>>(
        &self,
        plugin_id: Uuid,
        event_name: S,
        args: T,
    ) -> Result<(), ServiceError> {
        self.event_trigger_service.to_safe_fp()(plugin_id, event_name, args)
    }

    pub fn register_endpoint<
        S: Into<CString>,
        T: Into<CString>,
        Q: Into<CString>,
        F: RequestHandlerFunc,
    >(
        &self,
        args_schema: S,
        response_schema: T,
        plugin_id: Uuid,
        endpoint_name: Q,
    ) -> Result<(), ServiceError> {
        self.endpoint_register_service.to_safe_fp::<_, _, _, F>()(
            args_schema,
            response_schema,
            plugin_id,
            endpoint_name,
        )
    }

    pub fn unregister_endpoint<S: Into<CString>>(
        &self,
        plugin_id: Uuid,
        endpoint_name: S,
    ) -> Result<(), ServiceError> {
        self.endpoint_unregister_service.to_safe_fp()(plugin_id, endpoint_name)
    }

    pub fn endpoint_request<S: Into<CString>, T: Into<CString>>(
        &self,
        endpoint_name: S,
        args: T,
    ) -> Result<EndpointResponse, ServiceError> {
        self.endpoint_request_service.to_safe_fp()(endpoint_name, args)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_unsafe(
        handler_register_service: EventHandlerRegisterServiceUnsafeFP,
        handler_unregister_service: EventHandlerUnregisterServiceUnsafeFP,
        event_register_service: EventRegisterServiceUnsafeFP,
        event_unregister_service: EventUnregisterServiceUnsafeFP,
        event_trigger_service: EventTriggerServiceUnsafeFP,
        endpoint_register_service: EndpointRegisterServiceUnsafeFP,
        endpoint_unregister_service: EndpointUnregisterServiceUnsafeFP,
        endpoint_request_service: EndpointRequestServiceUnsafeFP,
    ) -> Self {
        Self {
            handler_register_service,
            handler_unregister_service,
            event_register_service,
            event_unregister_service,
            event_trigger_service,
            endpoint_register_service,
            endpoint_unregister_service,
            endpoint_request_service,
        }
    }
    pub fn new<
        HR: EventHandlerRegisterService,
        HU: EventHandlerUnregisterService,
        ER: EventRegisterService,
        EU: EventUnregisterService,
        ET: EventTriggerService,
        NR: EndpointRegisterService,
        NU: EndpointUnregisterService,
        NT: EndpointRequestService,
    >() -> Self {
        Self {
            handler_register_service: HR::adapter_fp(),
            handler_unregister_service: HU::adapter_fp(),
            event_register_service: ER::adapter_fp(),
            event_unregister_service: EU::adapter_fp(),
            event_trigger_service: ET::adapter_fp(),
            endpoint_register_service: NR::adapter_fp(),
            endpoint_unregister_service: NU::adapter_fp(),
            endpoint_request_service: NT::adapter_fp(),
        }
    }
}

impl From<ApplicationContext> for CApplicationContext {
    fn from(value: ApplicationContext) -> Self {
        value.to_c()
    }
}

pub struct PluginInfo {
    name: CString,
    version: CString,
    dependencies: CList_String,
    init_handler: EventHandlerFuncUnsafeFP,
    api_version: CApiVersion,
}

impl CPluginInfo {
    pub fn to_rust(self) -> Result<PluginInfo, ServiceError> {
        Ok(PluginInfo {
            name: self.name,
            version: self.version,
            dependencies: self.dependencies,
            init_handler: self.initHandler.err_null_fp()?,
            api_version: self.apiVersion,
        })
    }
}

impl PluginInfo {
    pub fn to_c(self) -> CPluginInfo {
        CPluginInfo {
            name: self.name,
            version: self.version,
            dependencies: self.dependencies,
            initHandler: Some(self.init_handler),
            apiVersion: self.api_version,
        }
    }

    pub fn new<E: EventHandlerFunc, N: Into<CString>, V: Into<CString>, D: Into<CList_String>>(
        name: N,
        version: V,
        dependencies: D,
        api_version: CApiVersion,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            dependencies: dependencies.into(),
            init_handler: E::adapter_fp(),
            api_version,
        }
    }

    pub fn new_unsafe<N: Into<CString>, V: Into<CString>, D: Into<CList_String>>(
        name: N,
        version: V,
        dependencies: D,
        handler: EventHandlerFuncUnsafeFP,
        api_version: CApiVersion,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            dependencies: dependencies.into(),
            init_handler: handler,
            api_version,
        }
    }

    pub fn name(&self) -> Result<&str, ApiMiscError> {
        self.name.as_str()
    }

    pub fn version(&self) -> Result<&str, ApiMiscError> {
        self.name.as_str()
    }

    pub fn dependencies(&self) -> Result<Vec<&str>, ApiMiscError> {
        self.dependencies.as_array()
    }

    pub fn handle<C: ContextSupplier, S: Into<CString>>(&self, context: C, args: S) {
        self.init_handler.to_safe_fp()(context, args)
    }

    pub fn handler(&self) -> EventHandlerFuncUnsafeFP {
        self.init_handler
    }

    pub fn api_version(&self) -> CApiVersion {
        self.api_version
    }
}

impl From<PluginInfo> for CPluginInfo {
    fn from(value: PluginInfo) -> Self {
        value.to_c()
    }
}

impl From<CPluginInfo> for Result<PluginInfo, ServiceError> {
    fn from(value: CPluginInfo) -> Self {
        value.to_rust()
    }
}

#[derive(Debug, Clone, Copy, Display, Error)]
pub enum ServiceError {
    CoreInternalError,
    NullFunctionPointer,
    InvalidString,
    InvalidJson,
    InvalidSchema,
    InvalidApi,
    NotFound,
    Unauthorized,
    Duplicate,
    PluginUninit,
    ShutingDown,
}

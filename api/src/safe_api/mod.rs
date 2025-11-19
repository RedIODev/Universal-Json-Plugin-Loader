use std::fmt::Debug;
use std::hash::Hash;

use derive_more::Display;
use thiserror::Error;
use uuid::Uuid;

pub mod pointer_traits;

use crate::cbindings::CApiVersion;
use crate::cbindings::{CApplicationContext, CList_String, CPluginInfo};
use crate::cbindings::{CEventHandler, CServiceError, CString};
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

///
/// This trait should be implemented by types where an error state can be converted into a `Result` of some type `T` and an appropriate `ServiceError`.
/// When converting into a `Result<T,ServiceError>` using the `error` function the implementation should print an error message to `std::error` when `debug_assertions are enabled`.
/// This trait is mainly design as an extension trait for Result and Option. To conveniently convert various errors into a debug print and the appropriate Api `ServiceError`. 
/// 
pub trait ErrorMapper<T> {
    ///
    /// Takes self and the requested `ServiceError` and converts it into a `Result` of the value `T` and the requested error.
    /// Like in the trait definition mention this function should print details to `std::error` when `debug_assertions` are enabled.
    /// 
    /// # Errors
    /// When the value of self cannot be converted into a value of `T` the function returns the supplied `ServiceError`.
    /// 
    fn error(self, error: ServiceError) -> Result<T, ServiceError>;
}

impl<T> ErrorMapper<T> for Option<T> {
    #[inline(always)]
    #[allow(clippy::inline_always)]
    fn error(self, error: ServiceError) -> Result<T, ServiceError> {
        #[cfg(debug_assertions)]
        if self.is_none() {
            use cli_colors::Colorizer;

            let colorizer = Colorizer::new();
            let line = colorizer.blue(format!("[Debug]{error}: Option<{}> is None.", std::any::type_name::<T>()));
            eprintln!("{line}");
        }
        self.ok_or(error)
    }
}

impl<T, E> ErrorMapper<T> for Result<T, E>
where
    E: std::fmt::Debug,
{
    #[inline(always)]
    #[allow(clippy::inline_always)]
    fn error(self, error: ServiceError) -> Result<T, ServiceError> {
        #[cfg(debug_assertions)]
        if let Err(ref e) = self {
            use cli_colors::Colorizer;

            let colorizer = Colorizer::new();
            let line = colorizer.blue(format!("[Debug]{error}: Result<{}, {}> is Err:",
                    std::any::type_name::<T>(),
                    std::any::type_name::<E>()));
            let error = colorizer.italic(format!("{e:?}"));
            eprintln!("{line}\n{error}");
        }
        self.ok().ok_or(error)
    }
}

impl CServiceError {
    ///
    /// Converts a `CServiceError` to the equivalent `ServiceError`.
    /// The `CServiceError::Success` variant is matched to `Ok(())`, every other variant is mapped to the equivalent Error as the `Err(..)` variant.
    /// # Errors
    /// When the self is not the `Success` variant a `ServiceError` is issued as an `Err`.
    /// 
    pub const fn to_rust(self) -> Result<(), ServiceError> {
        Err(match self {
            CServiceError::Success => return Ok(()),
            CServiceError::CoreInternalError => ServiceError::CoreInternalError,
            CServiceError::PluginInternalError => ServiceError::PluginInternalError,
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
    ///
    /// Converts a `ServiceError` to the equivalent `CServiceError`.
    /// 
    #[must_use]
    pub const fn to_c(self) -> CServiceError {
        match self {
            ServiceError::CoreInternalError => CServiceError::CoreInternalError,
            ServiceError::PluginInternalError => CServiceError::PluginInternalError,
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
    fn from((): ()) -> Self {
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

///
/// `EventHandler` is a type that represents a handler function with unique identity.
/// # Identity
/// The identity of the type is defined by it's `handler_id`.
/// Different `EventHandlers` with the same function but different `handler_ids` are considered different.
/// Meanwhile different `EventHandlers` with the same `handler_id` but different functions are considered equal.
/// `EventHandler` instances are considered identifiers for the registered handler. They can be used to remove a registered Handler. 
/// Therefore they and their `handler_id` shouldn't be shared with different plugins.
/// 
#[derive(Clone, Copy, Debug, Eq)]
pub struct EventHandler {
    function: EventHandlerFuncUnsafeFP,
    handler_id: Uuid,
}

impl CEventHandler {
    ///
    /// Converts a `CEventHandler` to the equivalent `EventHandler`.
    /// # Errors
    /// The conversion might fail when the `CEventHandler` contains an error value. In which case the Error is returned.
    /// 
    pub fn to_rust(self) -> Result<EventHandler, ServiceError> {
        self.error.to_rust()?;
        let func = self.function.error(ServiceError::NullFunctionPointer)?;

        Ok(EventHandler {
            function: func,
            handler_id: self.handler_id.into(),
        })
    }
}

impl EventHandler {
    ///
    /// Converts an `EventHandler` to the equivalent `CEventHandler`.
    /// 
    #[must_use]
    pub fn to_c(self) -> CEventHandler {
        CEventHandler {
            function: Some(self.function),
            handler_id: self.handler_id.into(),
            error: CServiceError::Success,
        }
    }

    ///
    /// Creates a new `EventHandler` from an unsafe function Pointer and a `handler_id`.
    /// # Safety
    /// This creation is safe as long as the function pointer implementation is following the C-api correctly.
    /// 
    pub fn new_unsafe(function: EventHandlerFuncUnsafeFP, handler_id: Uuid) -> Self {
        Self {
            function,
            handler_id,
        }
    }

    ///
    /// Creates a new `EventHandler` from a `handler_id` and the function as a generic parameter.
    /// 
    #[must_use]
    pub fn new<E: EventHandlerFunc>(handler_id: Uuid) -> Self {
        Self {
            function: E::adapter_fp(),
            handler_id,
        }
    }

    ///
    /// Calls the event handler with the provided arguments.
    /// # Panics
    /// This call may panic if the handler implementation is not following the C-api correctly.
    /// 
    pub fn handle<C: ContextSupplier, S: Into<CString>>(&self, context_supplier: C, args: S) {
        self.function.to_safe_fp()(context_supplier, args);
    }

    ///
    /// Gets the handler function associated with this `EventHandler`.
    /// 
    #[must_use]
    pub fn handler(&self) -> EventHandlerFuncUnsafeFP {
        self.function
    }

    ///
    /// Gets the `handler_id` associated with this `EventHandler`.
    /// 
    #[must_use]
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

///
/// The `ApplicationContext` struct allows interacting with the plugin system. 
/// It may be stored by plugins during execution. It is supplied in every `EventHandler` and `RequestHandler`.
/// It allows registering events, handlers, endpoints, removing and calling them.
/// # Lifetime
/// The guaranteed lifetime of an `ApplicationContext` instance is from the event `core:init` until the end of the event `core:power`(successful shutdown/restart).
/// The pointers providing the implementation for the `ApplicationContext` might be invalidated after restart. 
/// # Errors
/// Additionally to the error conditions listed in the individual functions every function might fail because of internal errors in the plugin loader.
/// 
#[must_use]
pub struct ApplicationContext {
    handler_register: EventHandlerRegisterServiceUnsafeFP,
    handler_unregister: EventHandlerUnregisterServiceUnsafeFP,
    event_register: EventRegisterServiceUnsafeFP,
    event_unregister: EventUnregisterServiceUnsafeFP,
    event_trigger: EventTriggerServiceUnsafeFP,
    endpoint_register: EndpointRegisterServiceUnsafeFP,
    endpoint_unregister: EndpointUnregisterServiceUnsafeFP,
    endpoint_request: EndpointRequestServiceUnsafeFP,
}

impl CApplicationContext {
    ///
    /// Converts a `CApplicationContext` to the equivalent `ApplicationContext`.
    /// # Errors
    /// The conversion might fail when the `CApplicationContext` contains an invalid value. In which case the Error is returned.
    pub fn to_rust(self) -> Result<ApplicationContext, ServiceError> {
        use ServiceError::NullFunctionPointer;
        Ok(ApplicationContext {
            handler_register: self.handlerRegisterService.error(NullFunctionPointer)?,
            handler_unregister: self.handlerUnregisterService.error(NullFunctionPointer)?,
            event_register: self.eventRegisterService.error(NullFunctionPointer)?,
            event_unregister: self.eventUnregisterService.error(NullFunctionPointer)?,
            event_trigger: self.eventTriggerService.error(NullFunctionPointer)?,
            endpoint_register: self.endpointRegisterService.error(NullFunctionPointer)?,
            endpoint_unregister: self.endpointUnregisterService.error(NullFunctionPointer)?,
            endpoint_request: self.endpointRequestService.error(NullFunctionPointer)?,
        })
    }
}

impl ApplicationContext {
    ///
    /// Converts an `ApplicationContext` to the equivalent `CApplicationContext`.
    /// 
    #[must_use]
    pub fn to_c(self) -> CApplicationContext {
        CApplicationContext {
            handlerRegisterService: Some(self.handler_register),
            handlerUnregisterService: Some(self.handler_unregister),
            eventRegisterService: Some(self.event_register),
            eventUnregisterService: Some(self.event_unregister),
            eventTriggerService: Some(self.event_trigger),
            endpointRegisterService: Some(self.endpoint_register),
            endpointUnregisterService: Some(self.endpoint_unregister),
            endpointRequestService: Some(self.endpoint_request),
        }
    }

    ///
    /// Registers a new `EventHandler` to a given event.
    /// The function takes an `EventHandlerFunc` generic parameter, the registering plugins id and the name of the event.
    /// On success a new `EventHandler` instance is returned which can be used to unregister the `EventHandler` later.
    /// The name follows the format "<plugin-name>:<event-name>"
    /// # Errors
    /// One reason the registration might fail is that the `handler_id` was already 
    /// registered in which case the old value stays unchanged and the new registration fails.
    /// 
    pub fn register_event_handler<E: EventHandlerFunc, T: Into<CString>>(
        &self,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<EventHandler, ServiceError> {
        self.handler_register.to_safe_fp::<E,_>()(plugin_id, event_name)
    }

    ///
    /// Unregisters an `EventHandler` given its `handler_id`, the `plugin_id` which registered the handler and the events name which the handler is registered for.
    /// The name follows the format "<plugin-name>:<event-name>"
    /// # Errors
    /// The unregistration might fail because no handler with the id can be found
    /// or the given `plugin_id` wasn't used when registering the handler.
    /// 
    pub fn unregister_event_handler<S: Into<CString>>(
        &self,
        handler_id: Uuid,
        plugin_id: Uuid,
        event_name: S,
    ) -> Result<(), ServiceError> {
        self.handler_unregister.to_safe_fp()(handler_id, plugin_id, event_name)
    }

    ///
    /// Registers a new event.
    /// An event is a 1 to many broadcast without a return value.
    /// The function takes an `args_schema` a `json_schema` describing the valid arguments for the event,
    /// a `plugin_id` that owns the event, and the name of the new event. The event name will be prefixed by this plugins name.
    /// A new event registration triggers the "core:event" event to inform other plugins about the new event and it's schema.
    /// # Errors
    /// The registration might fail, when the schema is invalid, the name contains a ':', the plugin is not found,
    /// or the event name was already registered for this plugin.
    /// 
    pub fn register_event<S: Into<CString>, T: Into<CString>>(
        &self,
        args_schema: S,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<(), ServiceError> {
        self.event_register.to_safe_fp()(args_schema, plugin_id, event_name)
    }

    ///
    /// Unregisters an event given the `plugin_id` the event was registered for and the name of the event. 
    /// The `event_name` must be the full event name including the plugin prefix.
    /// # Errors
    /// The unregistration might fail because no event with such name was found
    /// or the given `plugin_id` wasn't used when registering the event.
    /// 
    pub fn unregister_event<S: Into<CString>>(
        &self,
        plugin_id: Uuid,
        event_name: S,
    ) -> Result<(), ServiceError> {
        self.event_unregister.to_safe_fp()(plugin_id, event_name)
    }

    ///
    /// Triggers an event.
    /// An event is a 1 to many broadcast without a return value.
    /// To trigger an event the caller must be the owner of the event. 
    /// This will be checked using the `plugin_id`. The full `event_name` in the format "<plugin-name>:<event-name>"
    /// and valid arguments for the event must be provided.
    /// 
    /// # Errors
    /// The trigger of an event might fail, when the arguments aren't valid according to the events schema, 
    /// the `event_name` could not be found, or the `plugin_id` is not the owner of the event.
    /// 
    pub fn trigger_event<S: Into<CString>, T: Into<CString>>(
        &self,
        plugin_id: Uuid,
        event_name: S,
        args: T,
    ) -> Result<(), ServiceError> {
        self.event_trigger.to_safe_fp()(plugin_id, event_name, args)
    }

    ///
    /// Registers a new endpoint.
    /// An endpoint is a 1 to 1 request with a return value.
    /// The function takes an request handler generic parameter, an `args_schema` a `json_schema` describing the valid arguments for the endpoint,
    /// a `response_schema` a `json_schema` describing the return valid of the endpoint,
    /// a `plugin_id` that owns the endpoint and the name of the endpoint. The endpoint name will be prefixed by this plugins name.
    /// A new endpoint registration triggers the "core:endpoint" event to inform other plugins about the new endpoint and it's schema.
    /// # Errors
    /// The registration of an endpoint might fail because the `endpoint_name` contained a ':' character, the schema aren't valid,
    /// the plugin was not found, or the endpoint name was already registered for the plugin.
    /// 
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
        self.endpoint_register.to_safe_fp::<_, _, _, F>()(
            args_schema,
            response_schema,
            plugin_id,
            endpoint_name,
        )
    }

    ///
    /// Unregisters an endpoint given the `plugin_id` the endpoint was registered for and the name of the endpoint.
    /// The `endpoint_name` must be the full endpoint name including the plugin prefix.
    /// # Errors
    /// The unregistration might fail because no endpoint with such name was found
    /// or the given `plugin_id` wasn't used when registering the endpoint.
    /// 
    pub fn unregister_endpoint<S: Into<CString>>(
        &self,
        plugin_id: Uuid,
        endpoint_name: S,
    ) -> Result<(), ServiceError> {
        self.endpoint_unregister.to_safe_fp()(plugin_id, endpoint_name)
    }

    ///
    /// Makes a request to the endpoint.
    /// An endpoint is a 1 to 1 request with a return value.
    /// In contrast to event's there are no restrictions who can call an endpoint. 
    /// The `plugin_id` is not transmitted to the request handler, but the name of requesting plugin is submitted to the handler.
    /// To make a request the caller has to provide the full name of the endpoint including the plugin prefix, their `plugin_id`
    /// and valid arguments for the endpoint. The result will be returned and is validated by the endpoint's response schema.
    /// # Errors
    /// The request might fail, when the arguments aren't valid for the according to the endpoints schema,
    /// the endpoint name could not be found or the response from the handler isn't valid according to the endpoints response schema.
    /// 
    pub fn endpoint_request<S: Into<CString>, T: Into<CString>>(
        &self,
        endpoint_name: S,
        plugin_id: Uuid,
        args: T,
    ) -> Result<String, ServiceError> {
        self.endpoint_request.to_safe_fp()(endpoint_name, plugin_id, args)
    }

    ///
    /// Creates a new `ApplicationContext` from a set of unsafe function pointers.
    /// # Safety
    /// This creation is safe as long as the function pointers implementations are following the C-api correctly.
    /// 
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
            handler_register: handler_register_service,
            handler_unregister: handler_unregister_service,
            event_register: event_register_service,
            event_unregister: event_unregister_service,
            event_trigger: event_trigger_service,
            endpoint_register: endpoint_register_service,
            endpoint_unregister: endpoint_unregister_service,
            endpoint_request: endpoint_request_service,
        }
    }

    ///
    /// Creates a new `ApplicationContext` from the functions passed as generic parameters.
    /// 
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
            handler_register: HR::adapter_fp(),
            handler_unregister: HU::adapter_fp(),
            event_register: ER::adapter_fp(),
            event_unregister: EU::adapter_fp(),
            event_trigger: ET::adapter_fp(),
            endpoint_register: NR::adapter_fp(),
            endpoint_unregister: NU::adapter_fp(),
            endpoint_request: NT::adapter_fp(),
        }
    }
}

impl From<ApplicationContext> for CApplicationContext {
    fn from(value: ApplicationContext) -> Self {
        value.to_c()
    }
}

///
/// The `PluginInfo` struct represents all important information of a newly registered Plugin.
/// It is returned from the main function of a plugin.
/// 
pub struct PluginInfo {
    name: CString,
    version: CString,
    dependencies: CList_String,
    init_handler: EventHandlerFuncUnsafeFP,
    api_version: CApiVersion,
}

impl CPluginInfo {
    ///
    /// Converts a `CPluginInfo` to the equivalent `PluginInfo`
    /// # Errors
    /// The conversion might fail when the function pointer of the init function is invalid.
    /// 
    pub fn to_rust(self) -> Result<PluginInfo, ServiceError> {
        Ok(PluginInfo {
            name: self.name,
            version: self.version,
            dependencies: self.dependencies,
            init_handler: self.initHandler.error(ServiceError::NullFunctionPointer)?,
            api_version: self.apiVersion,
        })
    }
}

impl PluginInfo {
    ///
    /// Converts an `PluginInfo` to the equivalent `CPluginInfo`
    /// 
    #[must_use]
    pub fn to_c(self) -> CPluginInfo {
        CPluginInfo {
            name: self.name,
            version: self.version,
            dependencies: self.dependencies,
            initHandler: Some(self.init_handler),
            apiVersion: self.api_version,
        }
    }

    ///
    /// Creates a new `PluginInfo` from the init function as a generic parameter, the name of the plugin,
    /// the version of the plugin, its dependencies and the api version the plugin was compiled with.
    /// 
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

    ///
    /// Creates a new `PluginInfo` from the name of the plugin, the version of the plugin, its dependencies 
    /// the init function, and the api version the plugin was compiled with.
    /// # Safety
    /// This creation is safe as long as the function pointer implementation is following the C-api correctly.
    /// 
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

    ///
    /// A getter function for the name of a plugin.
    /// # Errors
    /// Getting the name might fail if the string is not valid.
    /// 
    pub fn name(&self) -> Result<&str, ApiMiscError> {
        self.name.as_str()
    }

    ///
    /// A getter function for the version of a plugin.
    /// # Errors
    /// Getting the version might fail if the string is not valid.
    /// 
    pub fn version(&self) -> Result<&str, ApiMiscError> {
        self.version.as_str()
    }

    ///
    /// A getter for the dependencies of a plugin.
    /// # Errors
    /// Getting the dependencies might fail if the strings or the list itself is not valid.
    /// 
    pub fn dependencies(&self) -> Result<Vec<&str>, ApiMiscError> {
        self.dependencies.as_array()
    }

    ///
    /// Calls the init function of this plugin with the given arguments.
    /// This requires a `ContextSupplier` and valid arguments for the "core:init" event.
    /// 
    pub fn handle<C: ContextSupplier, S: Into<CString>>(&self, context: C, args: S) {
        self.init_handler.to_safe_fp()(context, args);
    }

    ///
    /// A getter for the init function pointer for a plugin.
    /// 
    #[must_use]
    pub fn handler(&self) -> EventHandlerFuncUnsafeFP {
        self.init_handler
    }

    ///
    /// A getter for the api version a plugin is using.
    /// 
    #[must_use]
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

///
/// `ServiceError` represents all errors that can be reported from 
/// all calls through the C-api.
/// It differs to its C counterpart by not having a Success variant.
/// This is because the `CServiceError` is mapped to a `Result<(), ServiceError>`
/// and the Success variant is mapped to the Ok(()) Result.
/// 
#[derive(Debug, Clone, Copy, Display, Error)]
pub enum ServiceError {
    ///
    /// This variant signals that there was a problem in the core of the plugin loader.
    /// This should not be issued by plugins. Use `PluginInternalError` instead.
    /// 
    CoreInternalError,
    ///
    /// This variant signals that there was an internal problem in a plugin
    /// that is not further specified or doesn't fit the available errors.
    /// 
    PluginInternalError,
    ///
    /// This variant signals that a function pointer that was expected to be valid,
    /// was invalid or null.
    /// 
    NullFunctionPointer,
    ///
    /// This variant signals that a string was not valid. This can either be the string itself
    /// being invalid or its contents not being valid in the given context.
    /// 
    InvalidString,
    ///
    /// This variant signals that a string that was expected to be a json object was not a valid
    /// json string.
    /// 
    InvalidJson,
    ///
    /// This variant signals that a json object expected to be a `json_schema` was not a valid schema.
    /// 
    InvalidSchema,
    ///
    /// This variant signals that arguments or return values validated by a `json_schema` where not valid.
    /// 
    InvalidApi,
    ///
    /// This variant signals that something was not found. This could be a `plugin_id` lookup or an `event_name`
    /// that was not found, or similar cases.
    /// 
    NotFound,
    ///
    /// This variant signals that some plugin tried to do some action like triggering an event registered to a
    /// different plugin.
    /// 
    Unauthorized,
    ///
    /// This variant signals that some identifier was already registered. For example trying to register an event with the same name
    /// for the same plugin twice.
    /// 
    Duplicate,
    ///
    /// This variant is unused. Todo: remove variant!
    /// 
    PluginUninit,
    ///
    /// This variant signals that the plugin loader is currently shutting down or restarting 
    /// and a requested service was therefore not available.
    /// 
    ShutingDown,
}

pub mod pointer_traits;

use core::{
    any,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use derive_more::Display;
use thiserror::Error;
use uuid::Uuid;

use crate::cbindings::{
    CApiVersion, CApplicationContext, CEventHandler, CList_String, CPluginInfo, CServiceError,
    CString,
};
use crate::misc::ApiMiscError;
use crate::safe_api::pointer_traits::{
    ContextSupplier, EndpointRegisterService, EndpointRegisterServiceFPAdapter as _,
    EndpointRegisterServiceUnsafeFP, EndpointRequestService, EndpointRequestServiceFPAdapter as _,
    EndpointRequestServiceUnsafeFP, EndpointUnregisterService, EndpointUnregisterServiceUnsafeFP,
    EventHandlerFunc, EventHandlerFuncFPAdapter as _, EventHandlerFuncUnsafeFP,
    EventHandlerRegisterService, EventHandlerRegisterServiceFPAdapter as _,
    EventHandlerRegisterServiceUnsafeFP, EventHandlerUnregisterService,
    EventHandlerUnregisterServiceFPAdapter as _, EventHandlerUnregisterServiceUnsafeFP,
    EventRegisterService, EventRegisterServiceFPAdapter as _, EventRegisterServiceUnsafeFP,
    EventTriggerService, EventTriggerServiceFPAdapter as _, EventTriggerServiceUnsafeFP,
    EventUnregisterService, EventUnregisterServiceFPAdapter as _, EventUnregisterServiceUnsafeFP,
    RequestHandlerFunc,
};

///
/// This trait should be implemented by types where an error state can be converted into a `Result` of some type `T` and an appropriate `ServiceError`.
///
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
    #[expect(
        clippy::inline_always,
        clippy::print_stderr,
        reason = "config assertion wrapper. Should be inlined."
    )]
    fn error(self, error: ServiceError) -> Result<T, ServiceError> {
        #[cfg(debug_assertions)]
        if self.is_none() {
            use cli_colors::Colorizer;

            let colorizer = Colorizer::new();
            let line = colorizer.blue(format!(
                "[Debug]{error}: Option<{}> is None.",
                any::type_name::<T>()
            ));
            eprintln!("{line}");
        }
        self.ok_or(error)
    }
}

impl<T, E> ErrorMapper<T> for Result<T, E>
where
    E: Debug,
{
    #[inline(always)]
    #[expect(
        clippy::inline_always,
        clippy::print_stderr,
        reason = "config assertion wrapper. Should be inlined."
    )]
    fn error(self, error: ServiceError) -> Result<T, ServiceError> {
        #[cfg(debug_assertions)]
        if let Err(error_val) = &self {
            use cli_colors::Colorizer;

            let colorizer = Colorizer::new();
            let line = colorizer.blue(format!(
                "[Debug]{error}: Result<{}, {}> is Err:",
                any::type_name::<T>(),
                any::type_name::<E>()
            ));
            let error_msg = colorizer.italic(format!("{error_val:?}"));
            eprintln!("{line}\n{error_msg}");
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
    #[inline]
    pub const fn to_rust(self) -> Result<(), ServiceError> {
        Err(match self {
            Self::Success => return Ok(()),
            Self::CoreInternalError => ServiceError::CoreInternalError,
            Self::PluginInternalError => ServiceError::PluginInternalError,
            Self::NullFunctionPointer => ServiceError::NullFunctionPointer,
            Self::InvalidString => ServiceError::InvalidString,
            Self::InvalidJson => ServiceError::InvalidJson,
            Self::InvalidSchema => ServiceError::InvalidSchema,
            Self::InvalidApi => ServiceError::InvalidApi,
            Self::NotFound => ServiceError::NotFound,
            Self::Unauthorized => ServiceError::Unauthorized,
            Self::Duplicate => ServiceError::Duplicate,
            Self::PluginUninit => ServiceError::PluginUninit,
            Self::ShutingDown => ServiceError::ShutingDown,
        })
    }
}

impl ServiceError {
    ///
    /// Converts a `ServiceError` to the equivalent `CServiceError`.
    ///
    #[must_use]
    #[inline]
    pub const fn to_c(self) -> CServiceError {
        match self {
            Self::CoreInternalError => CServiceError::CoreInternalError,
            Self::PluginInternalError => CServiceError::PluginInternalError,
            Self::NullFunctionPointer => CServiceError::NullFunctionPointer,
            Self::InvalidString => CServiceError::InvalidString,
            Self::InvalidJson => CServiceError::InvalidJson,
            Self::InvalidSchema => CServiceError::InvalidSchema,
            Self::InvalidApi => CServiceError::InvalidApi,
            Self::NotFound => CServiceError::NotFound,
            Self::Unauthorized => CServiceError::Unauthorized,
            Self::Duplicate => CServiceError::Duplicate,
            Self::PluginUninit => CServiceError::PluginUninit,
            Self::ShutingDown => CServiceError::ShutingDown,
        }
    }
}

impl From<ServiceError> for CServiceError {
    #[inline]
    fn from(value: ServiceError) -> Self {
        value.to_c()
    }
}

impl From<()> for CServiceError {
    #[inline]
    fn from((): ()) -> Self {
        Self::Success
    }
}

impl From<Result<(), ServiceError>> for CServiceError {
    #[inline]
    fn from(value: Result<(), ServiceError>) -> Self {
        match value {
            Ok(ok) => ok.into(),
            Err(error) => error.into(),
        }
    }
}

impl From<CServiceError> for Result<(), ServiceError> {
    #[inline]
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
    #[inline]
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
    /// Calls the event handler with the provided arguments.
    /// # Panics
    /// This call may panic if the handler implementation is not following the C-api correctly.
    ///
    #[inline]
    pub fn handle<C: ContextSupplier, S: Into<CString>>(&self, context_supplier: C, args: S) {
        self.function.to_safe_fp()(context_supplier, args);
    }

    ///
    /// Gets the handler function associated with this `EventHandler`.
    ///
    #[must_use]
    #[inline]
    pub fn handler(&self) -> EventHandlerFuncUnsafeFP {
        self.function
    }

    ///
    /// Gets the `handler_id` associated with this `EventHandler`.
    ///
    #[must_use]
    #[inline]
    pub const fn id(&self) -> Uuid {
        self.handler_id
    }

    ///
    /// Creates a new `EventHandler` from a `handler_id` and the function as a generic parameter.
    ///
    #[must_use]
    #[inline]
    pub fn new<E: EventHandlerFunc>(handler_id: Uuid) -> Self {
        Self {
            function: E::adapter_fp(),
            handler_id,
        }
    }

    ///
    /// Creates a new `EventHandler` from an unsafe function Pointer and a `handler_id`.
    /// # Safe
    /// This creation is safe as long as the function pointer implementation is following the C-api correctly.
    ///
    #[inline]
    pub fn new_unsafe(function: EventHandlerFuncUnsafeFP, handler_id: Uuid) -> Self {
        Self {
            function,
            handler_id,
        }
    }

    ///
    /// Converts an `EventHandler` to the equivalent `CEventHandler`.
    ///
    #[must_use]
    #[inline]
    pub fn to_c(self) -> CEventHandler {
        CEventHandler {
            function: Some(self.function),
            handler_id: self.handler_id.into(),
            error: CServiceError::Success,
        }
    }
}

impl PartialEq for EventHandler {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.handler_id == other.handler_id
    }
}

impl Hash for EventHandler {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.handler_id.hash(state);
    }
}

impl From<EventHandler> for CEventHandler {
    #[inline]
    fn from(value: EventHandler) -> Self {
        value.to_c()
    }
}

impl From<ServiceError> for CEventHandler {
    #[inline]
    fn from(value: ServiceError) -> Self {
        Self::new_error(value.into())
    }
}

impl From<Result<EventHandler, ServiceError>> for CEventHandler {
    #[inline]
    fn from(value: Result<EventHandler, ServiceError>) -> Self {
        match value {
            Ok(ok) => ok.into(),
            Err(error) => error.into(),
        }
    }
}

impl From<CEventHandler> for Result<EventHandler, ServiceError> {
    #[inline]
    fn from(value: CEventHandler) -> Self {
        value.to_rust()
    }
}

///
/// The `ApplicationContext` struct allows interacting with the plugin system.
///
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
    endpoint_register: EndpointRegisterServiceUnsafeFP,
    endpoint_request: EndpointRequestServiceUnsafeFP,
    endpoint_unregister: EndpointUnregisterServiceUnsafeFP,
    event_handler_register: EventHandlerRegisterServiceUnsafeFP,
    event_handler_unregister: EventHandlerUnregisterServiceUnsafeFP,
    event_register: EventRegisterServiceUnsafeFP,
    event_trigger: EventTriggerServiceUnsafeFP,
    event_unregister: EventUnregisterServiceUnsafeFP,
}

impl CApplicationContext {
    ///
    /// Converts a `CApplicationContext` to the equivalent `ApplicationContext`.
    /// # Errors
    /// The conversion might fail when the `CApplicationContext` contains an invalid value. In which case the Error is returned.
    #[inline]
    pub fn to_rust(self) -> Result<ApplicationContext, ServiceError> {
        use ServiceError::NullFunctionPointer;
        Ok(ApplicationContext {
            event_handler_register: self.handlerRegisterService.error(NullFunctionPointer)?,
            event_handler_unregister: self.handlerUnregisterService.error(NullFunctionPointer)?,
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
    #[inline]
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
    /// # Safe
    ///
    /// Creates a new `ApplicationContext` from the functions passed as generic parameters.
    ///
    #[inline]
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
            event_handler_register: HR::adapter_fp(),
            event_handler_unregister: HU::adapter_fp(),
            event_register: ER::adapter_fp(),
            event_unregister: EU::adapter_fp(),
            event_trigger: ET::adapter_fp(),
            endpoint_register: NR::adapter_fp(),
            endpoint_unregister: NU::adapter_fp(),
            endpoint_request: NT::adapter_fp(),
        }
    }

    /// This creation is safe as long as the function pointers implementations are following the C-api correctly.
    ///
    #[expect(
        clippy::too_many_arguments,
        reason = "needs all handler as arguments for construction."
    )]
    #[inline]
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
            event_handler_register: handler_register_service,
            event_handler_unregister: handler_unregister_service,
            event_register: event_register_service,
            event_unregister: event_unregister_service,
            event_trigger: event_trigger_service,
            endpoint_register: endpoint_register_service,
            endpoint_unregister: endpoint_unregister_service,
            endpoint_request: endpoint_request_service,
        }
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
    #[inline]
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
    /// Registers a new event.
    /// An event is a 1 to many broadcast without a return value.
    /// The function takes an `args_schema` a `json_schema` describing the valid arguments for the event,
    /// a `plugin_id` that owns the event, and the name of the new event. The event name will be prefixed by this plugins name.
    /// A new event registration triggers the "core:event" event to inform other plugins about the new event and it's schema.
    /// # Errors
    /// The registration might fail, when the schema is invalid, the name contains a ':', the plugin is not found,
    /// or the event name was already registered for this plugin.
    ///
    #[inline]
    pub fn register_event<S: Into<CString>, T: Into<CString>>(
        &self,
        args_schema: S,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<(), ServiceError> {
        self.event_register.to_safe_fp()(args_schema, plugin_id, event_name)
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
    #[inline]
    pub fn register_event_handler<E: EventHandlerFunc, T: Into<CString>>(
        &self,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<EventHandler, ServiceError> {
        self.event_handler_register.to_safe_fp::<E, _>()(plugin_id, event_name)
    }

    ///
    /// Converts an `ApplicationContext` to the equivalent `CApplicationContext`.
    ///
    #[must_use]
    #[inline]
    pub fn to_c(self) -> CApplicationContext {
        CApplicationContext {
            handlerRegisterService: Some(self.event_handler_register),
            handlerUnregisterService: Some(self.event_handler_unregister),
            eventRegisterService: Some(self.event_register),
            eventUnregisterService: Some(self.event_unregister),
            eventTriggerService: Some(self.event_trigger),
            endpointRegisterService: Some(self.endpoint_register),
            endpointUnregisterService: Some(self.endpoint_unregister),
            endpointRequestService: Some(self.endpoint_request),
        }
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
    #[inline]
    pub fn trigger_event<S: Into<CString>, T: Into<CString>>(
        &self,
        plugin_id: Uuid,
        event_name: S,
        args: T,
    ) -> Result<(), ServiceError> {
        self.event_trigger.to_safe_fp()(plugin_id, event_name, args)
    }

    ///
    /// Unregisters an endpoint given the `plugin_id` the endpoint was registered for and the name of the endpoint.
    /// The `endpoint_name` must be the full endpoint name including the plugin prefix.
    /// # Errors
    /// The unregistration might fail because no endpoint with such name was found
    /// or the given `plugin_id` wasn't used when registering the endpoint.
    ///
    #[inline]
    pub fn unregister_endpoint<S: Into<CString>>(
        &self,
        plugin_id: Uuid,
        endpoint_name: S,
    ) -> Result<(), ServiceError> {
        self.endpoint_unregister.to_safe_fp()(plugin_id, endpoint_name)
    }

    ///
    /// Unregisters an event given the `plugin_id` the event was registered for and the name of the event.
    /// The `event_name` must be the full event name including the plugin prefix.
    /// # Errors
    /// The unregistration might fail because no event with such name was found
    /// or the given `plugin_id` wasn't used when registering the event.
    ///
    #[inline]
    pub fn unregister_event<S: Into<CString>>(
        &self,
        plugin_id: Uuid,
        event_name: S,
    ) -> Result<(), ServiceError> {
        self.event_unregister.to_safe_fp()(plugin_id, event_name)
    }

    ///
    /// Unregisters an `EventHandler` given its `handler_id`, the `plugin_id` which registered the handler and the events name which the handler is registered for.
    /// The name follows the format "<plugin-name>:<event-name>"
    /// # Errors
    /// The unregistration might fail because no handler with the id can be found
    /// or the given `plugin_id` wasn't used when registering the handler.
    ///
    #[inline]
    pub fn unregister_event_handler<S: Into<CString>>(
        &self,
        handler_id: Uuid,
        plugin_id: Uuid,
        event_name: S,
    ) -> Result<(), ServiceError> {
        self.event_handler_unregister.to_safe_fp()(handler_id, plugin_id, event_name)
    }
}

impl From<ApplicationContext> for CApplicationContext {
    #[inline]
    fn from(value: ApplicationContext) -> Self {
        value.to_c()
    }
}

///
/// The `PluginInfo` struct represents all important information of a newly registered Plugin.
/// It is returned from the main function of a plugin.
///
pub struct PluginInfo {
    api_version: CApiVersion,
    dependencies: CList_String,
    init_handler: EventHandlerFuncUnsafeFP,
    name: CString,
    version: CString,
}

impl CPluginInfo {
    ///
    /// Converts a `CPluginInfo` to the equivalent `PluginInfo`
    /// # Errors
    /// The conversion might fail when the function pointer of the init function is invalid.
    ///
    #[inline]
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
    /// A getter for the api version a plugin is using.
    ///
    #[must_use]
    #[inline]
    pub const fn api_version(&self) -> CApiVersion {
        self.api_version
    }

    ///
    /// A getter for the dependencies of a plugin.
    /// # Errors
    /// Getting the dependencies might fail if the strings or the list itself is not valid.
    ///
    #[inline]
    pub fn dependencies(&self) -> Result<Vec<&str>, ApiMiscError> {
        self.dependencies.as_array()
    }

    ///
    /// Calls the init function of this plugin with the given arguments.
    /// This requires a `ContextSupplier` and valid arguments for the "core:init" event.
    ///
    #[inline]
    pub fn handle<C: ContextSupplier, S: Into<CString>>(&self, context: C, args: S) {
        self.init_handler.to_safe_fp()(context, args);
    }

    ///
    /// A getter for the init function pointer for a plugin.
    ///
    #[must_use]
    #[inline]
    pub fn handler(&self) -> EventHandlerFuncUnsafeFP {
        self.init_handler
    }

    ///
    /// A getter function for the name of a plugin.
    /// # Errors
    /// Getting the name might fail if the string is not valid.
    ///
    #[inline]
    pub fn name(&self) -> Result<&str, ApiMiscError> {
        self.name.as_str()
    }

    ///
    /// Creates a new `PluginInfo` from the init function as a generic parameter, the name of the plugin,
    /// the version of the plugin, its dependencies and the api version the plugin was compiled with.
    ///
    #[inline]
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
    /// # Safe
    /// This creation is safe as long as the function pointer implementation is following the C-api correctly.
    ///
    #[inline]
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
    /// Converts an `PluginInfo` to the equivalent `CPluginInfo`
    ///
    #[must_use]
    #[inline]
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
    /// A getter function for the version of a plugin.
    /// # Errors
    /// Getting the version might fail if the string is not valid.
    ///
    #[inline]
    pub fn version(&self) -> Result<&str, ApiMiscError> {
        self.version.as_str()
    }
}

impl From<PluginInfo> for CPluginInfo {
    #[inline]
    fn from(value: PluginInfo) -> Self {
        value.to_c()
    }
}

impl From<CPluginInfo> for Result<PluginInfo, ServiceError> {
    #[inline]
    fn from(value: CPluginInfo) -> Self {
        value.to_rust()
    }
}

///
/// `ServiceError` represents all errors that can be reported from
/// all calls through the C-api.
///
/// It differs to its C counterpart by not having a Success variant.
/// This is because the `CServiceError` is mapped to a `Result<(), ServiceError>`
/// and the Success variant is mapped to the Ok(()) Result.
///
#[derive(Debug, Clone, Copy, Display, Error)]
#[non_exhaustive]
pub enum ServiceError {
    ///
    /// This variant signals that there was a problem in the core of the plugin loader.
    /// This should not be issued by plugins. Use `PluginInternalError` instead.
    ///
    CoreInternalError,
    ///
    /// This variant signals that some identifier was already registered. For example trying to register an event with the same name
    /// for the same plugin twice.
    ///
    Duplicate,
    ///
    /// This variant signals that arguments or return values validated by a `json_schema` where not valid.
    ///
    InvalidApi,
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
    /// This variant signals that a string was not valid. This can either be the string itself
    /// being invalid or its contents not being valid in the given context.
    ///
    InvalidString,
    ///
    /// This variant signals that something was not found. This could be a `plugin_id` lookup or an `event_name`
    /// that was not found, or similar cases.
    ///
    NotFound,
    ///
    /// This variant signals that a function pointer that was expected to be valid,
    /// was invalid or null.
    ///
    NullFunctionPointer,
    ///
    /// This variant signals that there was an internal problem in a plugin
    /// that is not further specified or doesn't fit the available errors.
    ///
    PluginInternalError,
    ///
    /// This variant is unused. Todo: remove variant!
    ///
    PluginUninit,
    ///
    /// This variant signals that the plugin loader is currently shutting down or restarting
    /// and a requested service was therefore not available.
    ///
    ShutingDown,
    ///
    /// This variant signals that some plugin tried to do some action like triggering an event registered to a
    /// different plugin.
    ///
    Unauthorized,
}

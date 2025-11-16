use std::{borrow::Cow, sync::Arc, time::Duration};

use chrono::{SecondsFormat, Utc};
use finance_together_api::{
    ApplicationContext, EndpointResponse, ErrorMapper, ServiceError,
    pointer_traits::{
        EndpointRegisterService, EndpointRequestService, EndpointUnregisterService,
        EventTriggerService, RequestHandlerFunc, RequestHandlerFuncFPAdapter,
        RequestHandlerFuncUnsafeFP, trait_fn,
    },
};
use im::HashMap;
use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::json;

use uuid::Uuid;

use crate::{
    config::ConfigRequestHandler, governor::get_gov, runtime::{ContextSupplierImpl, EventTrigger, PowerState, schema_from_file}, util::LockedMap
};

use ServiceError::CoreInternalError;

pub type Endpoints = LockedMap<Box<str>, Endpoint>;

#[derive(Clone)]
pub struct Endpoint {
    request_handler: RequestHandlerFuncUnsafeFP,
    argument_validator: Arc<Validator>,
    response_validator: Arc<Validator>,
    plugin_id: Uuid,
}

impl Endpoint {
    fn new(
        request_handler: RequestHandlerFuncUnsafeFP,
        argument_validator: Validator,
        response_validator: Validator,
        plugin_id: Uuid,
    ) -> Self {
        Self {
            request_handler,
            argument_validator: Arc::new(argument_validator),
            response_validator: Arc::new(response_validator),
            plugin_id,
        }
    }
}

pub fn register_core_endpoints(endpoints: &Endpoints, core_id: Uuid) {
    let mut new_endpoints = HashMap::new();
    new_endpoints.insert(
        "core:power".into(),
        Endpoint::new(
            CorePowerHandler::adapter_fp(),
            schema_from_file(include_str!("../../endpoint/power-args.json")),
            schema_from_file(include_str!("../../endpoint/power-resp.json")),
            core_id
        ),
    );
    new_endpoints.insert(
        "core:config".into(),
        Endpoint::new(
            ConfigRequestHandler::adapter_fp(),
            schema_from_file(include_str!("../../endpoint/config-args.json")),
            schema_from_file(include_str!("../../endpoint/config-resp.json")),
            core_id
        )
    );
    endpoints.rcu(|map| HashMap::clone(map).union(new_endpoints.clone()));
}

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum PowerCommand {
    Shutdown,
    Restart,
    Cancel,
}

#[derive(Deserialize)]
struct PowerArgs {
    command: PowerCommand,
    delay: Option<u32>,
}

#[trait_fn(RequestHandlerFunc for CorePowerHandler)]
pub fn handle<'a, F: Fn() -> ApplicationContext, S: Into<Cow<'a, str>>, T: AsRef<str>>(
    context_supplier: F,
    _: T,
    args: S,
) -> Result<EndpointResponse, ServiceError> {
    let args = serde_json::from_str::<PowerArgs>(&args.into()).error(ServiceError::InvalidJson)?;
    match get_gov().error(CoreInternalError)?.runtime().check_power() {
        PowerState::Shutdown | PowerState::Restart => return Err(ServiceError::ShutingDown),
        _ => {}
    }
    let core_id = get_gov().error(CoreInternalError)?.runtime().core_id();
    let context = context_supplier();
    let utc_now = Utc::now();
    let timestamp = utc_now.to_rfc3339_opts(SecondsFormat::Nanos, true);
    if let Some(delay) = args.delay {
        context.trigger_event(
            core_id,
            "core:power",
            json!({
                    "command": args.command,
                    "timestamp": timestamp,
                    "delay": delay
            })
            .to_string(),
        )?;
    } else {
        context.trigger_event(
            core_id,
            "core:power",
            json!({
                    "command": args.command,
                    "timestamp": timestamp
            })
            .to_string(),
        )?;
    }

    if let Some(delay) = args.delay {
        std::thread::sleep(Duration::from_millis(delay.into()));
    }

    if let PowerState::Cancel = get_gov().error(CoreInternalError)?.runtime().check_and_reset_power() {
        return Ok(EndpointResponse::new(json!({"canceled": true}).to_string()));
    }

    let power_state = match args.command {
        PowerCommand::Shutdown => PowerState::Shutdown,
        PowerCommand::Restart => PowerState::Restart,
        PowerCommand::Cancel => PowerState::Cancel,
    };

    get_gov().error(CoreInternalError)?.runtime().set_power(power_state);
    Ok(EndpointResponse::new(json!({}).to_string()))
}

#[trait_fn(EndpointRegisterService for EndpointRegister)]
pub(super) fn register<S: AsRef<str>, T: AsRef<str>, Q: AsRef<str>>(
    args_schema: S,
    response_schema: T,
    plugin_id: Uuid,
    endpoint_name: Q,
    handler: RequestHandlerFuncUnsafeFP,
) -> Result<(), ServiceError> {
    if endpoint_name.as_ref().contains(':') {
        return Err(ServiceError::InvalidString);
    }
    let argument_schema_json = serde_json::from_str(args_schema.as_ref()).error(ServiceError::InvalidJson)?;
    let argument_validator =
        jsonschema::validator_for(&argument_schema_json).error(ServiceError::InvalidSchema)?;
    let response_schema_json = serde_json::from_str(response_schema.as_ref()).error(ServiceError::InvalidJson)?;
    let response_validator =
        jsonschema::validator_for(&response_schema_json).error(ServiceError::InvalidSchema)?;
    let endpoint = Endpoint::new(handler, argument_validator, response_validator, plugin_id);
    let full_name = {
        let gov = get_gov().error(CoreInternalError)?;
        let plugins = gov.loader().plugins().load();
        let plugin_name = plugins.get(&plugin_id).map(|p| &*p.name).error(ServiceError::NotFound)?;
        if gov.endpoints().load().contains_key(endpoint_name.as_ref()) {
            return Err(ServiceError::Duplicate);
        }
        let full_name = format!("{}:{}", plugin_name, endpoint_name.as_ref());
        gov.endpoints()
            .rcu(|map| map.update(full_name.clone().into(), endpoint.clone()));
        full_name
    };

    let core_id = get_gov().error(CoreInternalError)?.runtime().core_id();
    EventTrigger::trigger(
        core_id,
        "core:endpoint",
        json!({
            "endpoint_name": full_name,
            "argument_schema": argument_schema_json,
            "response_schema": response_schema_json
        })
        .to_string(),
    )?;
    Ok(())
}

#[trait_fn(EndpointUnregisterService for EndpointUnregister)]
pub(super) fn unregister<S: AsRef<str>>(
    plugin_id: Uuid,
    endpoint_name: S,
) -> Result<(), ServiceError> {
    {
        let gov = get_gov().error(CoreInternalError)?;
        let endpoints = gov.endpoints().load();
        let endpoint = endpoints
            .get(endpoint_name.as_ref())
            .error(ServiceError::NotFound)?;
        if endpoint.plugin_id != plugin_id {
            return Err(ServiceError::Unauthorized);
        }

        gov.endpoints()
            .rcu(|map| map.without(endpoint_name.as_ref()));
    }

    Ok(())
}

#[trait_fn(EndpointRequestService for EndpointRequest)]
pub(super) fn request<'a, S: AsRef<str>, T: Into<Cow<'a, str>>>(
    endpoint_name: S,
    plugin_id: Uuid,
    args: T,
) -> Result<EndpointResponse, ServiceError> {
    let args = args.into();
    let arguments_json = serde_json::from_str(args.as_ref()).error(ServiceError::InvalidJson)?;
    let plugin_name = {
        let gov = get_gov().error(CoreInternalError)?;
        let plugins = gov.loader().plugins().load();
        plugins.get(&plugin_id)
                .map(|p| &*p.name)
                .error(ServiceError::NotFound)?
                .to_string()
    };
    let handler = {
        let gov = get_gov().error(CoreInternalError)?;
        let endpoints = gov.endpoints().load();
        let endpoint = endpoints
            .get(endpoint_name.as_ref())
            .error(ServiceError::NotFound)?;
        endpoint
            .argument_validator
            .validate(&arguments_json)
            .error(ServiceError::InvalidApi)?;
        endpoint.request_handler.to_safe_fp()
    };
    let response = handler(ContextSupplierImpl, plugin_name, args)?;

    let response_json =
        serde_json::from_str(response.response().error(ServiceError::InvalidString)?).error(ServiceError::InvalidJson)?;

    {
        let gov = get_gov().error(CoreInternalError)?;
        let endpoints = gov.endpoints().load();
        let endpoint = endpoints
            .get(endpoint_name.as_ref())
            .error(ServiceError::NotFound)?;
        endpoint
            .response_validator
            .validate(&response_json)
            .error(ServiceError::InvalidApi)?;
    }
    Ok(response)
}

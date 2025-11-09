use std::{sync::Arc, time::Duration};

use chrono::{SecondsFormat, Utc};
use finance_together_api::safe_api::{ApplicationContext, EndpointResponse, ServiceError, misc::OkOrCoreInternalError, pointer_traits::{EndpointRegisterService, EndpointRequestService, EndpointUnregisterService, EventTriggerService, RequestHandlerFunc, RequestHandlerFuncToSafe, RequestHandlerFuncUnsafeFP}};
use im::HashMap;
use jsonschema::Validator;
use serde::Deserialize;
use serde_json::json;
use trait_fn::trait_fn;
use uuid::Uuid;

use crate::{
    governor::get_gov,
    runtime::{ContextSupplierImpl, EventTrigger, PowerState, schema_from_file}, util::LockedMap,
};

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
            CorePowerHandler::unsafe_fp(),
            schema_from_file(include_str!("../../endpoint/power-args.json")),
            schema_from_file(include_str!("../../endpoint/power-resp.json")),
            core_id,
        ),
    );
    endpoints.rcu(|map| HashMap::clone(map).union(new_endpoints.clone()));
}

#[derive(Deserialize)]
struct PowerArgs {
    command: PowerState,
    delay: Option<u32>,
}


#[trait_fn(RequestHandlerFunc)]
pub fn CorePowerHandler
<F: Fn() -> ApplicationContext, S: AsRef<str>>
(context_supplier:F, args: S) -> Result<EndpointResponse, ServiceError> {
    let args = serde_json::from_str::<PowerArgs>(args.as_ref())
            .map_err(|_| ServiceError::InvalidInput0)?;
    match get_gov().ok_or_core()?.runtime().check_power() {
        PowerState::Shutdown | PowerState::Restart => return Err(ServiceError::ShutingDown),
            _ => {}
    }
    let core_id = get_gov().ok_or_core()?.runtime().core_id();
    let context = context_supplier();
    let utc_now = Utc::now();
    let timestamp = utc_now.to_rfc3339_opts(SecondsFormat::Nanos, true);
    context.trigger_event(
        core_id, 
        "core:power",  
        json!({
                "command": args.command,
                "timestamp": timestamp
        }).to_string()
    )?;

    if let Some(delay) = args.delay {
        std::thread::sleep(Duration::from_millis(delay as u64));
    }

    if let PowerState::Cancel = get_gov().ok_or_core()?.runtime().check_and_reset_power() {
        return Ok(EndpointResponse::new(json!({"canceled": true}).to_string()));
    }

    get_gov().ok_or_core()?.runtime().set_power(args.command);
    Ok(EndpointResponse::new(json!({}).to_string()))
}

// unsafe extern "C" fn core_power_handler(
//     context: ContextSupplier,
//     args: CString,
// ) -> EndpointResponse {
//     //lock endpoint to be called at most
//     let Ok(args) = args.as_str() else {
//         return EndpointResponse::new_error(ServiceError::InvalidInput0);
//     };
//     let Ok(args) = serde_json::from_str::<PowerArgs>(args) else {
//         return EndpointResponse::new_error(ServiceError::InvalidInput0);
//     };
//     let core_id;
//     {
//         // Mutex start
//         let Ok(gov) = get_gov() else {
//             return EndpointResponse::new_error(ServiceError::CoreInternalError);
//         };
//         match gov.runtime().check_power() {
//             PowerState::Shutdown | PowerState::Restart => return EndpointResponse::new_error(ServiceError::ShutingDown),
//             _ => {}
//         }
//         core_id = gov.runtime().core_id();
//     } // Mutex end
//     let Some(context) = context else {
//         return EndpointResponse::new_error(ServiceError::CoreInternalError);
//     };
//     let context = unsafe { context() };
//     let Some(trigger_service) = context.eventTriggerService else {
//         return EndpointResponse::new_error(ServiceError::CoreInternalError);
//     };
//     let utc_now = Utc::now();
//     let timestamp = utc_now.to_rfc3339_opts(SecondsFormat::Nanos, true);
//     let error = unsafe {
//         trigger_service(
//             core_id,
//             "core:power".into(),
//             json!({
//                 "command": args.command,
//                 "timestamp": timestamp
//             }).to_string().into(),
//         )
//     };
//     if let Err(error) = error.result() {
//         return EndpointResponse::new_error(error);
//     }
//     if let Some(delay) = args.delay {
//         std::thread::sleep(Duration::from_millis(delay as u64));
//     }

//     {
//         // Mutex start
//         let Ok(gov) = get_gov() else {
//             return EndpointResponse::new_error(ServiceError::CoreInternalError);
//         };
//         if let PowerState::Cancel = gov.runtime().check_and_reset_power() {
//             return EndpointResponse {
//                 response: json!({"canceled": true}).to_string().into(),
//                 error: ServiceError::Success,
//             };
//         }
//         gov.runtime().set_power(args.command);
//     } // Mutex end

//     EndpointResponse {
//         response: json!({}).to_string().into(),
//         error: ServiceError::Success,
//     }
// }

#[trait_fn(EndpointRegisterService)]
pub(super) fn EndpointRegister
<S: AsRef<str>, T: AsRef<str>, Q: AsRef<str>>(
    args_schema: S,
    response_schema: T,
    plugin_id: Uuid,
    endpoint_name: Q,
    handler: RequestHandlerFuncUnsafeFP) -> Result<(), ServiceError> {
    let argument_schema_json = serde_json::from_str(args_schema.as_ref())
        .map_err(|_| ServiceError::InvalidInput0)?;
    let argument_validator = jsonschema::validator_for(&argument_schema_json)
        .map_err(|_| ServiceError::InvalidInput0)?;
      let response_schema_json = serde_json::from_str(response_schema.as_ref())
        .map_err(|_| ServiceError::InvalidInput1)?;
    let response_validator = jsonschema::validator_for(&response_schema_json)
        .map_err(|_| ServiceError::InvalidInput1)?;  
    let endpoint = Endpoint::new(
        handler, 
        argument_validator, 
        response_validator, 
        plugin_id);
    let full_name = {
        let gov = get_gov().ok_or_core()?;
        let plugins = gov.loader().plugins().load();
        let plugin_name = plugins
            .get(&plugin_id)
            .map(|p| &*p.name)
            .ok_or_core()?;
        if gov.endpoints().load().contains_key(endpoint_name.as_ref()) {
            return Err(ServiceError::Duplicate);
        }
        let full_name = format!("{}:{}", plugin_name, endpoint_name.as_ref());
        gov.endpoints().rcu(
            |map| map.update(full_name.clone().into(), endpoint.clone()));
        full_name
    };


    let core_id = get_gov().ok_or_core()?.runtime().core_id();
    EventTrigger::safe(core_id, "core:endpoint", json!({
                "endpoint_name": full_name,
                "argument_schema": argument_schema_json,
                "response_schema": response_schema_json
            })
            .to_string())?;
    Ok(())
//     event_trigger(
//             core_id,
//             "core:endpoint".into(),
//             json!({
//                 "endpoint_name": full_name,
//                 "argument_schema": argument_schema,
//                 "response_schema": response_schema
//             })
//             .to_string()
//             .into()
//         )

}

// pub(super) unsafe extern "C" fn endpoint_register(
//     argument_schema: CString,
//     response_schema: CString,
//     plugin_id: CUuid,
//     endpoint_name: CString,
//     request_handler: CRequestHandlerFP,
// ) -> ServiceError {
//     let Ok(endpoint_name) = endpoint_name.as_str() else {
//         return ServiceError::InvalidInput3;
//     };

//     let Some(request_handler) = request_handler else {
//         return ServiceError::InvalidInput4;
//     };

//     let Ok(argument_schema) = argument_schema.as_str() else {
//         return ServiceError::InvalidInput0;
//     };
//     let Ok(argument_schema_json) = serde_json::from_str(argument_schema) else {
//         return ServiceError::InvalidInput0;
//     };
//     let Ok(argument_validator) = jsonschema::validator_for(&argument_schema_json) else {
//         return ServiceError::InvalidInput0;
//     };

//     let Ok(response_schema) = response_schema.as_str() else {
//         return ServiceError::InvalidInput1;
//     };
//     let Ok(response_schema_json) = serde_json::from_str(&response_schema) else {
//         return ServiceError::InvalidInput1;
//     };
//     let Ok(response_validator) = jsonschema::validator_for(&response_schema_json) else {
//         return ServiceError::InvalidInput1;
//     };

//     let endpoint = Endpoint::new(
//         request_handler,
//         argument_validator,
//         response_validator,
//         plugin_id,
//     );
//     let full_name;
//     let core_id = {
//         // Mutex start
//         let Ok(gov) = get_gov() else {
//             return ServiceError::CoreInternalError;
//         };

//         let Some(plugin_name) = gov
//             .loader()
//             .plugins()
//             .load()
//             .get(&plugin_id)
//             .map(|p| p.name.clone())
//         else {
//             return ServiceError::CoreInternalError;
//         };
//         let endpoints = gov.endpoints();
//         if endpoints.load().contains_key(endpoint_name) {
//             return ServiceError::Duplicate;
//         }
//         full_name = format!("{plugin_name}:{endpoint_name}");
//         endpoints.rcu(|map| map.update(full_name.clone().into(), endpoint.clone()));
//         gov.runtime().core_id()
//     }; // Mutex end

//     unsafe { 
//         event_trigger(
//             core_id,
//             "core:endpoint".into(),
//             json!({
//                 "endpoint_name": full_name,
//                 "argument_schema": argument_schema,
//                 "response_schema": response_schema
//             })
//             .to_string()
//             .into()
//         )
//     };
//     ServiceError::Success
// }
#[trait_fn(EndpointUnregisterService)]
pub(super) fn EndpointUnregister<S: AsRef<str>>(plugin_id: Uuid, endpoint_name: S) -> Result<(), ServiceError> {
    {
        let gov = get_gov().ok_or_core()?;
        let endpoints = gov.endpoints().load();
        let endpoint = endpoints.get(endpoint_name.as_ref())
                .ok_or(ServiceError::NotFound)?;
        if endpoint.plugin_id != plugin_id {
            return Err(ServiceError::Unauthorized);
        }

        gov.endpoints().rcu(|map| map.without(endpoint_name.as_ref()));
    }

    Ok(())
}

// pub(super) unsafe extern "C" fn endpoint_unregister(
//     plugin_id: CUuid,
//     endpoint_name: CString,
// ) -> ServiceError {
//     let Ok(endpoint_name) = endpoint_name.as_str() else {
//         return ServiceError::InvalidInput1;
//     };
//     {
//         // Mutex start
//         let Ok(gov) = get_gov() else {
//             return ServiceError::CoreInternalError;
//         };
//         let endpoints = gov.endpoints().load();
//         let Some(endpoint) = endpoints.get(endpoint_name) else {
//             return ServiceError::NotFound;
//         };
//         if endpoint.plugin_id != plugin_id {
//             return ServiceError::Unauthorized;
//         }
//         gov.endpoints().rcu(|map| map.without(endpoint_name));
//     } // Mutex end
//     ServiceError::Success
// }

#[trait_fn(EndpointRequestService)]
pub(super) fn EndpointRequest<S: AsRef<str>, T: AsRef<str>>(endpoint_name: S, args: T) -> Result<EndpointResponse, ServiceError> {
    let arguments_json = serde_json::from_str(args.as_ref())
            .map_err(|_| ServiceError::InvalidInput1)?;
    let handler = {
        let gov = get_gov().ok_or_core()?;
        let endpoints = gov.endpoints().load();
        let endpoint = endpoints.get(endpoint_name.as_ref())
            .ok_or(ServiceError::NotFound)?;
        endpoint.argument_validator.validate(&arguments_json)
                .map_err(|_| ServiceError::InvalidInput1)?;
        endpoint.request_handler.to_safe()
    };

    let response = handler(ContextSupplierImpl , args)?; //context supplier

    let response_json = serde_json::from_str(response.response()
                .map_err(|_| ServiceError::InvalidResponse)?)
            .map_err(|_| ServiceError::InvalidResponse)?;
    
    {
        let gov = get_gov().ok_or_core()?;
        let endpoints = gov.endpoints().load();
        let endpoint = endpoints.get(endpoint_name.as_ref()).ok_or(ServiceError::NotFound)?;
        endpoint.response_validator.validate(&response_json)
            .map_err(|_| ServiceError::InvalidResponse)?;
    }
    Ok(response)
}

// pub unsafe extern "C" fn endpoint_request(
//     endpoint_name: CString,
//     arguments: CString,
// ) -> EndpointResponse {
//     let Ok(endpoint_name) = endpoint_name.as_str() else {
//         return EndpointResponse::new_error(ServiceError::InvalidInput0);
//     };
//     let Ok(arguments) = arguments.as_str() else {
//         return EndpointResponse::new_error(ServiceError::InvalidInput1);
//     };
//     let handler;
//     {
//         // Mutex start
//         let Ok(gov) = get_gov() else {
//             return EndpointResponse::new_error(ServiceError::CoreInternalError);
//         };
//         let endpoints = gov.endpoints().load();
//         let Some(endpoint) = endpoints.get(endpoint_name) else {
//             return EndpointResponse::new_error(ServiceError::NotFound);
//         };
//         let Ok(arguments_json) = serde_json::from_str(arguments) else {
//             return EndpointResponse::new_error(ServiceError::InvalidInput1);
//         };
//         if let Err(_) = endpoint.argument_validator.validate(&arguments_json) {
//             return EndpointResponse::new_error(ServiceError::InvalidInput1);
//         }
//         handler = endpoint.request_handler;
//     } // Mutex end

//     let result = unsafe { handler(Some(context_supplier), arguments.into()) }; // might lock mutex
//     if result.error != ServiceError::Success {
//         return result;
//     }
//     let Ok(response) = result.response.as_str() else {
//         return EndpointResponse::new_error(ServiceError::InvalidResponse);
//     };
//     let Ok(response) = serde_json::from_str(response) else {
//         return EndpointResponse::new_error(ServiceError::InvalidResponse);
//     };

//     {
//         // Mutex start
//         let Ok(gov) = get_gov() else {
//             return EndpointResponse::new_error(ServiceError::CoreInternalError);
//         };
//         let endpoints = gov.endpoints().load();
//         let Some(endpoint) = endpoints.get(endpoint_name) else {
//             return EndpointResponse::new_error(ServiceError::NotFound);
//         };
//         if let Err(_) = endpoint.response_validator.validate(&response) {
//             return EndpointResponse::new_error(ServiceError::InvalidResponse);
//         }
//         result
//     } // Mutex end
// }

use std::{collections::hash_map::Entry, time::Duration};

use finance_together_api::{
    RequestHandlerFP,
    cbindings::{
        CRequestHandlerFP, CString, CUuid, ContextSupplier, EndpointResponse, ServiceError,
    },
};
use jsonschema::Validator;
use serde::Deserialize;
use serde_json::json;

use crate::{
    governor::{read_gov, write_gov, Endpoints}, runtime::{context_supplier, event::event_trigger, schema_from_file, PowerState}
};

pub struct Endpoint {
    request_handler: RequestHandlerFP,
    argument_validator: Validator,
    response_validator: Validator,
    plugin_id: CUuid,
}

impl Endpoint {
    fn new(
        request_handler: RequestHandlerFP,
        argument_validator: Validator,
        response_validator: Validator,
        plugin_id: CUuid,
    ) -> Self {
        Self {
            request_handler,
            argument_validator,
            response_validator,
            plugin_id,
        }
    }
}

pub fn register_core_endpoints(core_id: CUuid) -> Endpoints {
    let mut endpoints = Endpoints::new();
    endpoints.insert(
        "core:power".into(),
        Endpoint::new(
            core_power_handler,
            schema_from_file(include_str!("../../endpoint/power-args.json")),
            schema_from_file(include_str!("../../endpoint/power-resp.json")),
            core_id,
        ),
    );
    endpoints
}



#[derive(Deserialize)]
struct PowerArgs {
    command: PowerState,
    delay: Option<u32>,
}

unsafe extern "C" fn core_power_handler(
    context: ContextSupplier,
    args: CString,
) -> EndpointResponse { //lock endpoint to be called at most 
    let Ok(args) = args.as_str() else {
        return EndpointResponse::new_error(ServiceError::InvalidInput0);
    };
    let Ok(args) = serde_json::from_str::<PowerArgs>(args) else {
        return EndpointResponse::new_error(ServiceError::InvalidInput0);
    };
    let core_id;
    {
        // Mutex start
        let Ok(gov) = read_gov() else {
            return EndpointResponse::new_error(ServiceError::CoreInternalError);
        };
        core_id = gov.runtime().core_id();
    } // Mutex end
    let Some(context) = context else {
        return EndpointResponse::new_error(ServiceError::CoreInternalError);
    };
    let context = unsafe { context() };
    let Some(trigger_service) = context.eventTriggerService else {
        return EndpointResponse::new_error(ServiceError::CoreInternalError);
    };
    let error = unsafe {
        trigger_service(
            core_id,
            "core:power".into(),
            json!({"command": args.command}).to_string().into(),
        )
    };
    if let Err(error) = error.result() {
        return EndpointResponse::new_error(error);
    }
    if let Some(delay) = args.delay {
        std::thread::sleep(Duration::from_millis(delay as u64));
    }

    {
        // Mutex start
        let Ok(mut gov) = write_gov() else {
            return EndpointResponse::new_error(ServiceError::CoreInternalError);
        };
        if let Some(PowerState::Cancel) = gov.runtime_mut().check_and_reset_power() {
            return EndpointResponse {
                response: json!({"canceled": true}).to_string().into(),
                error: ServiceError::Success,
            };
        }
        gov.runtime_mut().set_power(args.command);
    } // Mutex end

    
    
    EndpointResponse { response: json!({}).to_string().into(), error: ServiceError::Success }
}

pub(super) unsafe extern "C" fn endpoint_register(
    argument_schema: CString,
    response_schema: CString,
    plugin_id: CUuid,
    endpoint_name: CString,
    request_handler: CRequestHandlerFP,
) -> ServiceError {
    let Ok(endpoint_name) = endpoint_name.as_str() else {
        return ServiceError::InvalidInput3;
    };

    let Some(request_handler) = request_handler else {
        return ServiceError::InvalidInput4;
    };

    let Ok(argument_schema) = argument_schema.as_str() else {
        return ServiceError::InvalidInput0;
    };
    let Ok(argument_schema) = serde_json::from_str(argument_schema) else {
        return ServiceError::InvalidInput0;
    };
    let Ok(argument_validator) = jsonschema::validator_for(&argument_schema) else {
        return ServiceError::InvalidInput0;
    };

    let Ok(response_schema) = response_schema.as_str() else {
        return ServiceError::InvalidInput1;
    };
    let Ok(response_schema) = serde_json::from_str(&response_schema) else {
        return ServiceError::InvalidInput1;
    };
    let Ok(response_validator) = jsonschema::validator_for(&response_schema) else {
        return ServiceError::InvalidInput1;
    };

    let endpoint = Endpoint {
        request_handler,
        argument_validator,
        response_validator,
        plugin_id,
    };

    let core_id = {
        // Mutex start
        let Ok(mut gov) = write_gov() else {
            return ServiceError::CoreInternalError;
        };

        let Some(plugin_name) = gov
            .loader()
            .plugins()
            .get(&plugin_id)
            .map(|p| p.name.clone())
        else {
            return ServiceError::CoreInternalError;
        };
        let endpoints = gov.endpoints_mut();
        if endpoints.contains_key(endpoint_name) {
            return ServiceError::Duplicate;
        }
        endpoints.insert(format!("{plugin_name}:{endpoint_name}").into(), endpoint);
        gov.runtime().core_id()
    }; // Mutex end

    unsafe { event_trigger(core_id, "core:endpoint".into(), format!("").into()) };
    ServiceError::Success
}

pub(super) unsafe extern "C" fn endpoint_unregister(
    plugin_id: CUuid,
    endpoint_name: CString,
) -> ServiceError {
    let Ok(endpoint_name) = endpoint_name.as_str() else {
        return ServiceError::InvalidInput1;
    };
    {
        // Mutex start
        let Ok(mut gov) = write_gov() else {
            return ServiceError::CoreInternalError;
        };
        let Entry::Occupied(o) = gov.endpoints_mut().entry(endpoint_name.into()) else {
            return ServiceError::NotFound;
        };
        if o.get().plugin_id != plugin_id {
            return ServiceError::Unauthorized;
        }
        o.remove();
    } // Mutex end
    ServiceError::Success
}

pub unsafe extern "C" fn endpoint_request(
    endpoint_name: CString,
    arguments: CString,
) -> EndpointResponse {
    let Ok(endpoint_name) = endpoint_name.as_str() else {
        return EndpointResponse::new_error(ServiceError::InvalidInput0);
    };
    let Ok(arguments) = arguments.as_str() else {
        return EndpointResponse::new_error(ServiceError::InvalidInput1);
    };
    let handler;
    {
        // Mutex start
        let Ok(gov) = read_gov() else {
            return EndpointResponse::new_error(ServiceError::CoreInternalError);
        };
        let Some(endpoint) = gov.endpoints().get(endpoint_name) else {
            return EndpointResponse::new_error(ServiceError::NotFound);
        };
        let Ok(arguments_json) = serde_json::from_str(arguments) else {
            return EndpointResponse::new_error(ServiceError::InvalidInput1);
        };
        if let Err(_) = endpoint.argument_validator.validate(&arguments_json) {
            return EndpointResponse::new_error(ServiceError::InvalidInput1);
        }
        handler = endpoint.request_handler;
    } // Mutex end

    let result = unsafe { handler(Some(context_supplier), arguments.into()) }; // might lock mutex
    if result.error != ServiceError::Success {
        return result;
    }
    let Ok(response) = result.response.as_str() else {
        return EndpointResponse::new_error(ServiceError::InvalidResponse);
    };
    let Ok(response) = serde_json::from_str(response) else {
        return EndpointResponse::new_error(ServiceError::InvalidResponse);
    };

    {
        // Mutex start
        let Ok(gov) = read_gov() else {
            return EndpointResponse::new_error(ServiceError::CoreInternalError);
        };
        let Some(endpoint) = gov.endpoints().get(endpoint_name) else {
            return EndpointResponse::new_error(ServiceError::NotFound);
        };
        if let Err(_) = endpoint.response_validator.validate(&response) {
            return EndpointResponse::new_error(ServiceError::InvalidResponse);
        }
        result
    } // Mutex end
}

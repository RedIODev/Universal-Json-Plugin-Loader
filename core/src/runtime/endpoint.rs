use std::collections::hash_map::Entry;

use finance_together_api::{
    RequestHandlerFP,
    cbindings::{CRequestHandlerFP, CString, CUuid, EndpointResponse, ServiceError},
};
use jsonschema::Validator;

use crate::{governor::Endpoints, runtime::{context_supplier, event::event_trigger}, GGL};

pub struct Endpoint {
    request_handler: RequestHandlerFP,
    argument_validator: Validator,
    response_validator: Validator,
    plugin_id: CUuid,
}

pub fn register_core_endpoints(core_id: CUuid) -> Endpoints {
    let endpoints = Endpoints::new();
    endpoints
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
        let Ok(mut gov) = GGL.write() else {
            return ServiceError::CoreInternalError;
        };

        let Some(plugin_name) = gov.plugins().get(&plugin_id).map(|p| p.name.clone()) else {
            return ServiceError::CoreInternalError;
        };
        let endpoints = gov.endpoints_mut();
        if endpoints.contains_key(endpoint_name) {
            return ServiceError::Duplicate;
        }
        endpoints.insert(format!("{plugin_name}:{endpoint_name}").into(), endpoint);
        gov.core_id()
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
        let Ok(mut gov) = GGL.write() else {
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
    {
        // Mutex start
        let Ok(gov) = GGL.read() else {
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
        let handler = endpoint.request_handler;
        let result = unsafe { handler(Some(context_supplier), arguments.into())};

        if result.error != ServiceError::Success {
            return result;
        }
        let Ok(response) = result.response.as_str() else {
            return EndpointResponse::new_error(ServiceError::InvalidResponse);
        };
        let Ok(response) = serde_json::from_str(response) else {
            return EndpointResponse::new_error(ServiceError::InvalidResponse);
        };
        if let Err(_) = endpoint.response_validator.validate(&response) {
            return EndpointResponse::new_error(ServiceError::InvalidResponse);
        }

        result
    } // Mutex end
}

use finance_together_api::cbindings::{CRequestHandlerFP, CString, CUuid, EndpointResponse, ServiceError};



pub(super) unsafe extern "C" fn endpoint_register(argument_schema: CString, response_schema: CString, plugin_id: CUuid, endpoint_name: CString, request_handler: CRequestHandlerFP) -> ServiceError {
    ServiceError::CoreInternalError
}

pub(super) unsafe extern "C" fn endpoint_unregister(plugin_id: CUuid, endpoint_name: CString) -> ServiceError {
    ServiceError::CoreInternalError
}

pub unsafe extern "C" fn endpoint_request(plugin_id: CUuid, endpoint_name: CString, arguments: CString) -> EndpointResponse {
    EndpointResponse { response: "".into(), error: ServiceError::CoreInternalError }
}
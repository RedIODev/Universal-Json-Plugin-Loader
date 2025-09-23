#ifndef FT_ENDPOINT_H
#define FT_ENDPOINT_H
#include "ft_string.h"
#include "ft_util.h"

// EndpointResponse struct that carries the success state and the response with it.
// All fields values are undefined unless the error field == SERVICE_SUCCESS.
typedef struct {
    String response;
    ServiceError error;
} EndpointResponse;

typedef EndpointResponse (*RequestHandlerFP)(NON_NULL ContextSupplier, String);

// Service function to register a new endpoint.
// The first    argument is the json schema the endpoints arguments have to satisfy.
// The second   argument is the json schema the response has to satisfy.
// The third    argument has to be the plugins uuid.
// The fourth   argument is the endpoints name. This will be prefixed by this plugins name.
// The fifth    argument is the endpoints handler function that handles the requests to the endpoint.
// Returns the success state of the registration.
typedef ServiceError (*EndpointRegisterService)(String, String, Uuid, String, RequestHandlerFP);

// Service function to unregister an endpoint.
// The first    argument has to be the plugins uuid.
// The second   argument is the endpoints name to be removed.
// Returns the success state of the unregistration.
typedef ServiceError (*EndpointUnregisterService)(Uuid, String);

// Service function to call an endpoint.
// The second   argument is the endpoint name to be called.
// The third    argument is the endpoints arguments.
// Returns the response of the endpoint.
typedef EndpointResponse (*EndpointRequestService)(String, String);

#endif
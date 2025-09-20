#ifndef FT_API_H 
#define FT_API_H

#include "ft_types.h"
#include "ft_string.h"
#include <stddef.h>

//forward declaration due to cycle
struct ApplicationContext;

typedef struct ApplicationContext (*ContextSupplier)();

// The Type used for handling messages.
// The first    argument is the supplier function for the Application Context.
// The second   argument is the input args from the event in the specified json format.
typedef void (*HandlerFP)(NON_NULL ContextSupplier, String);


// Uuid type to pass uuids over the ffi boundary safely
typedef struct {
    u64 first;
    u64 second;
} Uuid;

typedef enum {
    SUCCESS_SERVICE = 0,
    CORE_INTERNAL_ERROR_SERVICE,
    INVALID_INPUT_0_SERVICE,
    INVALID_INPUT_1_SERVICE,
    INVALID_INPUT_2_SERVICE,
    INVALID_INPUT_3_SERVICE,
    INVALID_INPUT_4_SERVICE,
    NOT_FOUND_SERVICE,
    UNAUTHORIZED_SERVICE,
    DUPLICATE_SERVICE,
    PLUGIN_UNINIT_SERVICE
} ServiceError;

// Handler struct that carries the success state and the generated handler_id with it.
// The handler_id is required to unregister the handler later.
// All fields values are undefined unless the error field == SUCCESS_SERVICE.
typedef struct {
    NON_NULL HandlerFP function;
    Uuid handler_id;
    ServiceError error;
} Handler;

// Service function to register a new handler for a given event.
// The first    argument is the handler to be registered.
// The second   argument has to be the plugins uuid.
// The third    argument is the events name to register the handler to. The name follows the format "<plugin-name>:<event-name>"
// Returns the success state of the registration.
typedef Handler (*HandlerRegisterService)(NON_NULL HandlerFP, Uuid, String);

// Service function to unregister a handler for a given event.
// The first    argument is the handlerId to be removed.
// The second   argument has to be the plugins uuid.
// The third    argument is the events name to remove the handler from. The name follows the format "<plugin-name>:<event-name>"
// Returns the success state of the unregistration.
typedef ServiceError (*HandlerUnregisterService)(Uuid, Uuid, String);

// Service function to register a new event.
// The first    argument is the json schema the events arguments have to satisfy.
// The second   argument has to be the plugins uuid.
// The third    argument is the events name. This will be prefixed by this plugins name.
// Returns the success state of the registration.
typedef ServiceError (*EventRegisterService)(String, Uuid, String);

// Service function to unregister an event.
// The first    argument has to be the plugins uuid.
// The second   argument is the events name to be removed.
// Returns the success state of the unregistration.
typedef ServiceError (*EventUnregisterService)(Uuid, String);

// Service function to trigger an event.
// The first    argument has to be the plugins uuid.
// The second   argument is the events name to be triggered.
// The third    argument is the events arguments.
// Returns the success state of the trigger.
typedef ServiceError (*EventTriggerService)(Uuid, String, String);

// Application context that provides configuration services for the plugin to interact with the core application.
// With a context you can register, uregister and trigger events and handlers.
typedef struct ApplicationContext {
    NON_NULL HandlerRegisterService handlerRegisterService; 
    NON_NULL HandlerUnregisterService HandlerUnregisterService;
    NON_NULL EventRegisterService eventRegisterService;
    NON_NULL EventUnregisterService EventUnregisterService;
    NON_NULL EventTriggerService eventTriggerService;
} ApplicationContext;

NON_NULL HandlerFP pluginMain(Uuid);

#endif
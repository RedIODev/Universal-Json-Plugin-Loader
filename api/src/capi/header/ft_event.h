#ifndef FT_EVENT_H
#define FT_EVENT_H

#include "ft_util.h"
#include "ft_string.h"

// The Type used for handling messages.
// The first    argument is the supplier function for the Application Context.
// The second   argument is the input args from the event in the specified json format.
typedef void (*EventHandlerFP)(NON_NULL ContextSupplier, String);

// Handler struct that carries the success state and the generated handler_id with it.
// The handler_id is required to unregister the handler later.
// All fields values are undefined unless the error field == SERVICE_SUCCESS.
typedef struct
{
    NON_NULL EventHandlerFP function;
    Uuid handler_id;
    ServiceError error;
} EventHandler;

// Service function to register a new handler for a given event.
// The first    argument is the handler to be registered.
// The second   argument has to be the plugins uuid.
// The third    argument is the events name to register the handler to. The name follows the format "<plugin-name>:<event-name>"
// Returns the success state of the registration.
typedef EventHandler (*EventHandlerRegisterService)(NON_NULL EventHandlerFP, Uuid, String);

// Service function to unregister a handler for a given event.
// The first    argument is the handlerId to be removed.
// The second   argument has to be the plugins uuid.
// The third    argument is the events name to remove the handler from. The name follows the format "<plugin-name>:<event-name>"
// Returns the success state of the unregistration.
typedef ServiceError (*EventHandlerUnregisterService)(Uuid, Uuid, String);

// Service function to register a new event.
// The first    argument is the json schema the events arguments have to satisfy.
// The second   argument has to be the plugins uuid.
// The third    argument is the events name. This will be prefixed by this plugins name. It can't contain any ':' characters.
// Returns the success state of the registration.
typedef ServiceError (*EventRegisterService)(String, Uuid, String);

// Service function to unregister an event.
// The first    argument has to be the plugins uuid.
// The second   argument is the events name to be removed. The name follows the format "<plugin-name>:<event-name>"
// Returns the success state of the unregistration.
typedef ServiceError (*EventUnregisterService)(Uuid, String);

// Service function to trigger an event.
// Events are triggered sequentially but don't block the triggering thread.
// Success is returned as soon as the event is scheduled successfully.
// The first    argument has to be the plugins uuid.
// The second   argument is the events name to be triggered.
// The third    argument is the events arguments.
// Returns the success state of the trigger.
typedef ServiceError (*EventTriggerService)(Uuid, String, String);

#endif
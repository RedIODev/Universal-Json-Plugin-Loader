#ifndef FT_API_H 
#define FT_API_H

#include "ft_types.h"
#include "ft_string.h"
#include <stddef.h>

//forward declaration due to cycle
struct ApplicationContext;

typedef struct ApplicationContext (*ContextSupplier)();

// The Type used for handling messages.
// The first String is the argument is the arguments of the message to be handled.
// The second argument is used as an output.
// If the handler finishes successfully it should return true and the output parameter contains the result.
// If the handler encounters an error handling the request it should return false and the output parameter is an error message.
typedef bool (*Handler)(NON_NULL ContextSupplier, String, OUT String *);

// Uuid type to pass uuids over the ffi boundary safely
typedef struct {
    u64 first;
    u64 second;
} Uuid;

// Service function to register a new handler for a given event.
// The first    argument is the handler to be registered.
// The second   argument has to be the plugins uuid.
// The third    argument is the events name to register the handler to. The name follows the format "<plugin-name>:<event-name>"
// returns true when the handler was added successfully, otherwise false.
typedef bool (*HandlerRegisterService)(NON_NULL Handler, Uuid, String);

// Service function to unregister a handler for a given event.
// The first    argument is the handler to be removed.
// The second   argument has to be the plugins uuid.
// The third    argument is the events name to remove the handler from. The name follows the format "<plugin-name>:<event-name>"
// returns true when the handler was removed successfully, false if it was not present or an error occurred.
typedef bool (*HandlerUnregisterService)(NON_NULL Handler, Uuid, String);

// Service function to register a new event.
// The first    argument is the json schema the events arguments have to satisfy.
// The second   argument is the json schema the events response has to satisfy.
// The third    argument has to be the plugins uuid.
// The forth    argument is the events name. This will be prefixed by this plugins name.
// returns true when the event was added successfully, otherwise false.
typedef bool (*EventRegisterService)(String, String, Uuid, String);

// Service function to unregister an event.
// The first    argument has to be the plugins uuid.
// The second   argument is the events name to be removed.
// returns true when the event was removed successfully, false if it was not present or an error occurred.
typedef bool (*EventUnregisterService)(Uuid, String);

// Application context that provides configuration services for the plugin to interact with the core application.
// With a context you can register and uregister events and handlers.
typedef struct ApplicationContext {
    NON_NULL HandlerRegisterService handlerRegisterService; 
    NON_NULL HandlerUnregisterService HandlerUnregisterService;
    NON_NULL EventRegisterService eventRegisterService;
    NON_NULL EventUnregisterService EventUnregisterService;
} ApplicationContext;

NON_NULL Handler pluginMain(Uuid);

#endif
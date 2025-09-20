#ifndef FT_API_H 
#define FT_API_H

#include "ft_types.h"
#include "ft_string.h"
#include <stddef.h>

typedef struct {
    int i;
} ApplicationContext;

typedef ApplicationContext (*ContextSupplier)();

// The Type used for handling messages.
// The first String is the argument is the arguments of the message to be handled.
// The second argument is used as an output.
// If the handler finishes successfully it should return true and the output parameter contains the result.
// If the handler encounters an error handling the request it should return false and the output parameter is an error message.
typedef bool (*Handler)(NON_NULL ContextSupplier, String, OUT String *);

typedef struct {
    u64 lower;
    u64 higher;
} Uuid;

typedef bool (*HandlerRegisterService)(NON_NULL Handler, Uuid, String);

void pluginMain(NON_NULL HandlerRegisterService, Uuid);

#endif
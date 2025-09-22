#ifndef FT_UTIL_H
#define FT_UTIL_H

#include "ft_types.h"

//forward declaration due to cycle
struct ApplicationContext;

typedef struct ApplicationContext (*ContextSupplier)();



// Uuid type to pass uuids over the ffi boundary safely
typedef struct {
    u64 first;
    u64 second;
} Uuid;

typedef enum {
    SERVICE_SUCCESS = 0,
    SERVICE_CORE_INTERNAL_ERROR,
    SERVICE_INVALID_INPUT_0,
    SERVICE_INVALID_INPUT_1,
    SERVICE_INVALID_INPUT_2,
    SERVICE_INVALID_INPUT_3,
    SERVICE_INVALID_INPUT_4,
    SERVICE_NOT_FOUND,
    SERVICE_UNAUTHORIZED,
    SERVICE_DUPLICATE,
    SERVICE_PLUGIN_UNINIT
} ServiceError;

#endif
#ifndef FT_UTIL_H
#define FT_UTIL_H

#include "ft_types.h"

//forward declaration due to cycle
struct ApplicationContext;

typedef struct ApplicationContext (*ContextSupplier)();

#define NULL_GUARD(var, ret) if (var == NULL) { \
    return ret;\
}

// Uuid type to pass uuids over the ffi boundary safely
typedef struct {
    u64 higher;
    u64 lower;
} Uuid;

typedef enum {
    SERVICE_SUCCESS = 0,
    SERVICE_CORE_INTERNAL_ERROR,
    SERVICE_PLUGIN_INTERNAL_ERROR,
    SERVICE_NULL_FUNCTION_POINTER,
    SERVICE_INVALID_STRING,
    SERVICE_INVALID_JSON,
    SERVICE_INVALID_SCHEMA,
    SERVICE_INVALID_API,
    SERVICE_NOT_FOUND,
    SERVICE_UNAUTHORIZED,
    SERVICE_DUPLICATE,
    SERVICE_PLUGIN_UNINIT,
    SERVICE_SHUTING_DOWN
} ServiceError;

#endif
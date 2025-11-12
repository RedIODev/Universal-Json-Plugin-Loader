#ifndef FT_API_H
#define FT_API_H

#include "ft_util.h"
#include "ft_event.h"
#include "ft_endpoint.h"
#include "ft_string.h"
#include <stddef.h>

// Application context that provides configuration services for the plugin to interact with the core application.
// With a context you can register, uregister and trigger events, endpoints and handlers.
typedef struct ApplicationContext
{
    NON_NULL EventHandlerRegisterService handlerRegisterService;
    NON_NULL EventHandlerUnregisterService handlerUnregisterService;
    NON_NULL EventRegisterService eventRegisterService;
    NON_NULL EventUnregisterService eventUnregisterService;
    NON_NULL EventTriggerService eventTriggerService;
    NON_NULL EndpointRegisterService endpointRegisterService;
    NON_NULL EndpointUnregisterService endpointUnregisterService;
    NON_NULL EndpointRequestService endpointRequestService;
} ApplicationContext;

typedef struct
{
    u16 major;
    u8 feature;
    u8 patch;
} ApiVersion;

typedef struct
{
    String name;
    String version;
    List_String dependencies;
    EventHandlerFP initHandler;
    ApiVersion apiVersion;
} PluginInfo;

extern const ApiVersion API_VERSION;
NON_NULL PluginInfo plugin_main(Uuid);

#endif
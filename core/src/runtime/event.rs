use std::{collections::HashSet, hash::Hash, sync::Arc};

use finance_together_api::{
    EventHandler, EventHandlerFP,
    cbindings::{CEventHandler, CEventHandlerFP, CString, CUuid, ServiceError},
};
use im::HashMap;
use jsonschema::Validator;
use serde_json::json;
use topo_sort::TopoSort;
use uuid::Uuid;

use crate::{
    governor::{Events, get_gov},
    loader::Plugin,
    runtime::{context_supplier, schema_from_file},
    util::{ArcMapExt, TrueOrErr},
};

#[derive(Clone)]
pub struct Event {
    pub handlers: HashSet<StoredEventHandler>,
    argument_validator: Arc<Validator>,
    plugin_id: CUuid,
}

impl Event {
    pub fn new(argument_validator: Validator, plugin_id: CUuid) -> Self {
        Self {
            handlers: HashSet::new(),
            argument_validator: Arc::new(argument_validator),
            plugin_id,
        }
    }
}

#[derive(Clone)]
pub struct StoredEventHandler {
    handler: EventHandler,
    plugin_id: CUuid,
}

impl Hash for StoredEventHandler {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.plugin_id.hash(state);
    }
}

impl PartialEq for StoredEventHandler {
    fn eq(&self, other: &Self) -> bool {
        self.plugin_id == other.plugin_id
    }
}

impl Eq for StoredEventHandler {}

impl StoredEventHandler {
    pub fn new(handler: EventHandler, plugin_id: CUuid) -> Self {
        Self { handler, plugin_id }
    }
}

pub fn register_core_events(events: &Events, core_id: CUuid) {
    let mut new_events = HashMap::new();
    new_events.insert(
        "core:init".into(),
        Event::new(
            schema_from_file(include_str!("../../event/init.json")),
            core_id,
        ),
    );
    new_events.insert(
        "core:event".into(),
        Event::new(
            schema_from_file(include_str!("../../event/event.json")),
            core_id,
        ),
    );
    new_events.insert(
        "core:endpoint".into(),
        Event::new(
            schema_from_file(include_str!("../../event/endpoint.json")),
            core_id,
        ),
    );
    new_events.insert(
        "core:power".into(),
        Event::new(
            schema_from_file(include_str!("../../event/power.json")),
            core_id,
        ),
    );
    events.rcu(|map| HashMap::clone(map).union(new_events.clone()));
}

pub(super) unsafe extern "C" fn handler_register(
    handler_fp: CEventHandlerFP,
    plugin_id: CUuid,
    event_name: CString,
) -> CEventHandler {
    let Some(function) = handler_fp else {
        return CEventHandler::new_error(ServiceError::InvalidInput0);
    };
    let Ok(event_name) = event_name.as_str() else {
        return CEventHandler::new_error(ServiceError::InvalidInput2);
    };
    let handler = EventHandler {
        function,
        handler_id: CUuid::from_u64_pair(Uuid::new_v4().as_u64_pair()),
    };
    {
        // Mutex start
        let Ok(gov) = get_gov() else {
            return CEventHandler::new_error(ServiceError::CoreInternalError);
        };

        let new_handler = StoredEventHandler {
            handler: handler.clone(),
            plugin_id,
        };

        let result = gov.events().rcu_alter(event_name, |event| {
            event
                .handlers
                .insert(new_handler.clone())
                .or_error(ServiceError::Duplicate)
        });

        if let Err(err) = result {
            return CEventHandler::new_error(err);
        }
    } // Mutex end

    handler.into()
}

pub(super) unsafe extern "C" fn handler_unregister(
    handler_id: CUuid,
    plugin_id: CUuid,
    event_name: CString,
) -> ServiceError {
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput2;
    };
    {
        // Mutex start
        let Ok(gov) = get_gov() else {
            return ServiceError::CoreInternalError;
        };
        gov.events()
            .rcu_alter(event_name, |event| {
                let handler = event
                    .handlers
                    .iter()
                    .find(|h| h.handler.handler_id == handler_id)
                    .ok_or(ServiceError::NotFound)?;

                if handler.plugin_id != plugin_id {
                    return Err(ServiceError::Unauthorized);
                }
                event.handlers.remove(&handler.clone());
                Ok(())
            })
            .err()
            .unwrap_or(ServiceError::Success)
    } // Mutex end
}

pub(super) unsafe extern "C" fn event_register(
    argument_schema: CString,
    plugin_id: CUuid,
    event_name: CString,
) -> ServiceError {
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput2;
    };
    let Ok(argument_schema) = argument_schema.as_str() else {
        return ServiceError::InvalidInput0;
    };
    let Ok(argument_schema_json) = serde_json::from_str(argument_schema) else {
        return ServiceError::InvalidInput0;
    };
    let Ok(validator) = jsonschema::validator_for(&argument_schema_json) else {
        return ServiceError::InvalidInput0;
    };
    let event = Event::new(validator, plugin_id);
    let full_name;
    let core_id = {
        // Mutex start
        let Ok(gov) = get_gov() else {
            return ServiceError::CoreInternalError;
        };
        let Some(plugin_name) = gov
            .loader()
            .plugins()
            .load()
            .get(&plugin_id)
            .map(|p| p.name.clone())
        else {
            return ServiceError::CoreInternalError;
        };
        let events = gov.events();
        if events.load().contains_key(event_name) {
            return ServiceError::Duplicate;
        }
        full_name = format!("{plugin_name}:{event_name}");

        events.rcu(|map| map.update(full_name.clone().into(), event.clone()));
        gov.runtime().core_id()
    }; // Mutex end
    unsafe {
        event_trigger(
            core_id,
            "core:event".into(),
            json!({"event_name": full_name, "argument_schema": argument_schema})
                .to_string()
                .into(),
        )
    }; // locks mutex
    ServiceError::Success
}

pub(super) unsafe extern "C" fn event_unregister(
    plugin_id: CUuid,
    event_name: CString,
) -> ServiceError {
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput1;
    };
    {
        // Mutex start
        let Ok(gov) = get_gov() else {
            return ServiceError::CoreInternalError;
        };
        let events = gov.events().load();
        let Some(event) = events.get(event_name) else {
            return ServiceError::NotFound;
        };
        if event.plugin_id != plugin_id {
            return ServiceError::Unauthorized;
        }

        gov.events().rcu(|events| events.without(event_name));
    } // Mutex end
    ServiceError::Success
}

pub unsafe extern "C" fn event_trigger(
    plugin_id: CUuid,
    event_name: CString,
    arguments: CString,
) -> ServiceError {
    let Ok(event_name) = event_name.as_str() else {
        return ServiceError::InvalidInput1;
    };
    let Ok(arguments) = arguments.as_str() else {
        return ServiceError::InvalidInput2;
    };
    {
        // Mutex start
        let Ok(gov) = get_gov() else {
            return ServiceError::CoreInternalError;
        };
        let events = gov.events().load();
        let Some(event) = events.get(event_name) else {
            return ServiceError::NotFound;
        };
        if event.plugin_id != plugin_id {
            return ServiceError::Unauthorized;
        }
        let Ok(json_arguments) = serde_json::from_str(arguments) else {
            return ServiceError::InvalidInput2;
        };
        if let Err(_) = event.argument_validator.validate(&json_arguments) {
            return ServiceError::InvalidInput2;
        }
        let funcs = if event_name != "core:init" {
            event.handlers.iter().map(|h| h.handler.function).collect()
        } else {
            let Some(funcs) = sort_handlers(event.handlers.iter(), &gov.loader().plugins().load())
            else {
                return ServiceError::CoreInternalError;
            };
            funcs
        };
        let arguments = arguments.to_owned();
        gov.runtime().event_pool.execute(move || {
            for func in funcs {
                unsafe { func(Some(context_supplier), arguments.clone().into()) } // might lock mutex
            }
        });
    }; // Mutex end

    ServiceError::Success
}

fn sort_handlers<'a>(
    handlers: impl Iterator<Item = &'a StoredEventHandler>,
    stored_plugins: &HashMap<CUuid, Plugin>,
) -> Option<Vec<EventHandlerFP>> {
    let nodes: Vec<_> = handlers
        .map(|handler| {
            TopoNode::create_dependency_entry(handler, || stored_plugins.values(), stored_plugins)
        })
        .collect::<Option<_>>()?;
    let mut sorter = TopoSort::new();
    for (node, deps) in nodes {
        sorter.insert_from_set(node, deps);
    }
    Some(
        sorter
            .iter()
            .map(|node| node.ok()?.0.1)
            .collect::<Option<_>>()?,
    )
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct TopoNode(CUuid, Option<EventHandlerFP>);

impl TopoNode {
    fn create_dependency_entry<'a, F, I>(
        handler: &StoredEventHandler,
        plugins: F,
        stored_plugins: &HashMap<CUuid, Plugin>,
    ) -> Option<(TopoNode, HashSet<TopoNode>)>
    where
        F: Fn() -> I,
        I: Iterator<Item = &'a Plugin>,
    {
        let plugin = stored_plugins.get(&handler.plugin_id)?;
        let node = TopoNode(handler.plugin_id, Some(handler.handler.function));
        let deps = plugin
            .dependencies
            .iter()
            .map(|dep| TopoNode::find_plugin_by_name(plugins(), dep))
            .map(|dep| TopoNode::find_id_for_plugin(dep?, stored_plugins))
            .collect::<Option<_>>()?;
        Some((node, deps))
    }

    fn find_plugin_by_name<'a>(
        mut plugins: impl Iterator<Item = &'a Plugin>,
        name: &str,
    ) -> Option<&'a Plugin> {
        plugins.find(|plugin| *plugin.name == *name)
    }

    fn find_id_for_plugin(
        plugin: &Plugin,
        stored_plugins: &HashMap<CUuid, Plugin>,
    ) -> Option<TopoNode> {
        stored_plugins.iter().find_map(|(key, value)| {
            if value.name == plugin.name {
                Some(TopoNode(*key, None))
            } else {
                None
            }
        })
    }
}

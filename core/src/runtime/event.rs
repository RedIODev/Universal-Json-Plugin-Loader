use std::{collections::HashSet, hash::Hash, sync::Arc};

use finance_together_api::{
    ErrorMapper, EventHandler, ServiceError, pointer_traits::{EventHandlerFuncToSafe, EventHandlerFuncUnsafeFP, EventRegisterService, EventTriggerService, EventUnregisterService, HandlerRegisterService, HandlerUnregisterService, trait_fn}
};
use im::HashMap;
use jsonschema::Validator;
use serde_json::json;
use topo_sort::TopoSort;
use uuid::Uuid;

use crate::{
    governor::get_gov,
    loader::Plugin,
    runtime::{ContextSupplierImpl, PowerState, schema_from_file},
    util::{ArcMapExt, LockedMap, TrueOrErr},
};

pub type Events = LockedMap<Box<str>, Event>;

#[derive(Clone)]
pub struct Event {
    pub handlers: HashSet<StoredEventHandler>,
    argument_validator: Arc<Validator>,
    plugin_id: Uuid,
}

impl Event {
    pub fn new(argument_validator: Validator, plugin_id: Uuid) -> Self {
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
    plugin_id: Uuid,
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
    pub fn new(handler: EventHandler, plugin_id: Uuid) -> Self {
        Self { handler, plugin_id }
    }
}

pub fn register_core_events(events: &Events, core_id: Uuid) {
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

#[trait_fn(HandlerRegisterService)]
pub(super) fn EventHandlerRegister
    <T: AsRef<str>>
    (handler: EventHandlerFuncUnsafeFP, plugin_id: Uuid, event_name: T) -> Result<EventHandler, ServiceError> {
    let event_handler = EventHandler::new_unsafe(handler, Uuid::new_v4());
    let stored_handler = StoredEventHandler::new(event_handler, plugin_id);

    get_gov().err_core()?.events().rcu_alter(event_name.as_ref(), |event| {
        event.handlers.insert(stored_handler.clone()).or_error(ServiceError::Duplicate)
    })?;

    Ok(event_handler)
}

#[trait_fn(HandlerUnregisterService)] 
pub(super) fn HandlerUnregister<S: AsRef<str>>(
        handler_id: Uuid,
        plugin_id: Uuid,
        event_name: S,
    ) -> Result<(), ServiceError> {
    get_gov().err_core()?.events().rcu_alter(event_name.as_ref(), |event| {
        let handler = event
                .handlers
                .iter()
                .find(|h| h.handler.id() == handler_id)
                .ok_or(ServiceError::NotFound)?;
        if handler.plugin_id != plugin_id {
            return Err(ServiceError::Unauthorized);
        }
        event.handlers.remove(&handler.clone());
        Ok(())
    })?;

    Ok(())
}

#[trait_fn(EventRegisterService)]
pub(super) fn EventRegister<S: AsRef<str>, T: AsRef<str>>(
        argument_schema: S,
        plugin_id: Uuid,
        event_name: T,
    ) -> Result<(), ServiceError> {
    let argument_schema_json = serde_json::from_str(argument_schema.as_ref())
            .err_invalid_json()?;

    let argument_validator = jsonschema::validator_for(&argument_schema_json)
            .err_invalid_schema()?;
    let event = Event::new(argument_validator, plugin_id);
    let full_name = {
        let gov = get_gov().err_core()?;
        let plugins = gov.loader().plugins().load();
        let plugin_name = plugins
            .get(&plugin_id)
            .map(|p| &*p.name)
            .err_not_found()?;
        if gov.events().load().contains_key(event_name.as_ref()) {
            return Err(ServiceError::Duplicate);
        }
        let full_name = format!("{}:{}", plugin_name, event_name.as_ref());
        gov.events().rcu(|map|map.update(full_name.clone().into(), event.clone()));
        full_name
    };

    let core_id = get_gov().err_core()?.runtime().core_id();

    EventTrigger::safe(core_id, 
        "core:event", 
        json!({
                    "event_name": full_name,
                    "argument_schema": argument_schema_json
                })
                .to_string())
}

#[trait_fn(EventUnregisterService)] 
pub(super) fn EventUnregister<S: AsRef<str>>(plugin_id: Uuid, event_name: S) -> Result<(), ServiceError> {
    {
        let gov = get_gov().err_core()?;
        let events = gov.events().load();
        let event = events.get(event_name.as_ref())
                .ok_or(ServiceError::NotFound)?;
        if event.plugin_id != plugin_id {
            return Err(ServiceError::Unauthorized);
        }

        gov.events().rcu(|events| events.without(event_name.as_ref()));
    }

    Ok(())
}

#[trait_fn(EventTriggerService)]
pub(super) fn EventTrigger<S: AsRef<str>, T: AsRef<str>>(
        plugin_id: Uuid,
        event_name: S,
        args: T,
    ) -> Result<(), ServiceError> {
    match get_gov().err_core()?.runtime().check_power() {
        PowerState::Shutdown | PowerState::Restart => return Err(ServiceError::ShutingDown),
        _ => {}
    }

    let event_arguments_json = serde_json::from_str(args.as_ref())
            .err_invalid_json()?;
    let funcs = {
        let gov = get_gov().err_core()?;
        let events = gov.events().load();
        let event = events.get(event_name.as_ref())
                .ok_or(ServiceError::NotFound)?;
        if event.plugin_id != plugin_id {
            return Err(ServiceError::Unauthorized);
        }

        event.argument_validator.validate(&event_arguments_json)
                .err_invalid_api()?;
        if event_name.as_ref() != "core:init" {
            event.handlers.iter().map(|h| h.handler.handler()).collect()
        } else {
            let Some(funcs) = sort_handlers(event.handlers.iter(), &gov.loader().plugins().load())
            else {
                return Err(ServiceError::CoreInternalError);
            };
            funcs
        }
    };
    let executor = get_gov().err_core()?.runtime().event_pool.clone();
    let owned_args = args.as_ref().to_string();
    executor.execute(move || {
        for func in funcs {
            func.to_safe()(ContextSupplierImpl, owned_args.clone());
        }
    });
    Ok(())
}

fn sort_handlers<'a>(
    handlers: impl Iterator<Item = &'a StoredEventHandler>,
    stored_plugins: &HashMap<Uuid, Plugin>,
) -> Option<Vec<EventHandlerFuncUnsafeFP>> {
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
struct TopoNode(Uuid, Option<EventHandlerFuncUnsafeFP>);

impl TopoNode {
    fn create_dependency_entry<'a, F, I>(
        handler: &StoredEventHandler,
        plugins: F,
        stored_plugins: &HashMap<Uuid, Plugin>,
    ) -> Option<(TopoNode, HashSet<TopoNode>)>
    where
        F: Fn() -> I,
        I: Iterator<Item = &'a Plugin>,
    {
        let plugin = stored_plugins.get(&handler.plugin_id)?;
        let node = TopoNode(handler.plugin_id, Some(handler.handler.handler()));
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
        stored_plugins: &HashMap<Uuid, Plugin>,
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

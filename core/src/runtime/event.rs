use std::{collections::HashSet};
use core::hash::{Hash, Hasher};
use plugin_loader_api::{
    ErrorMapper as _, EventHandler, ServiceError,
    pointer_traits::{
        EventHandlerFuncUnsafeFP, EventHandlerRegisterService,
        EventHandlerUnregisterService, EventRegisterService, EventTriggerService,
        EventUnregisterService, trait_fn,
    },
};
use im::HashMap;
use jsonschema::Validator;
use serde_json::json;
use topo_sort::TopoSort;
use uuid::Uuid;

use crate::{
    governor::get_gov,
    loader::Plugin,
    runtime::{ContextSupplierImpl, PowerState, RuntimeError, schema_from_file},
    util::{ArcMapExt as _, LockedMap, TrueOrErr as _},
};

use ServiceError::CoreInternalError;

pub type Events = LockedMap<Box<str>, Event>;

#[derive(Clone)]
pub struct Event {
    argument_validator: Validator,
    handlers: HashSet<StoredEventHandler>,
    plugin_id: Uuid,
}

impl Event {
    pub const fn handlers_mut(&mut self) -> &mut HashSet<StoredEventHandler> {
        &mut self.handlers
    }

    pub fn new(argument_validator: Validator, plugin_id: Uuid) -> Self {
        Self {
            handlers: HashSet::new(),
            argument_validator,
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
    fn hash<H: Hasher>(&self, state: &mut H) {
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
    pub const fn new(handler: EventHandler, plugin_id: Uuid) -> Self {
        Self { handler, plugin_id }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct TopoNode(Uuid, Option<EventHandler>);

impl TopoNode {
    #[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
    fn create_dependency_entry<'plugin, F, I>(
        handler: &StoredEventHandler,
        plugins: F,
        stored_plugins: &HashMap<Uuid, Plugin>,
    ) -> Option<(Self, HashSet<Self>)>
    where
        F: Fn() -> I,
        I: Iterator<Item = &'plugin Plugin>,
    {
        let plugin = stored_plugins.get(&handler.plugin_id)?;
        let node = Self(handler.plugin_id, Some(handler.handler));
        let deps = plugin
            .dependencies()
            .iter()
            .map(|dep| Self::find_plugin_by_name(plugins(), dep))
            .map(|dep| Self::find_id_for_plugin(dep?, stored_plugins))
            .collect::<Option<_>>()?;
        Some((node, deps))
    }

    #[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
    fn find_id_for_plugin(
        plugin: &Plugin,
        stored_plugins: &HashMap<Uuid, Plugin>,
    ) -> Option<Self> {
        stored_plugins.iter().find_map(|(key, value)| {
            (value.name() == plugin.name()).then_some(Self(*key, None))
        })
    }

    #[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
    fn find_plugin_by_name<'plugin>(
        mut plugins: impl Iterator<Item = &'plugin Plugin>,
        name: &str,
    ) -> Option<&'plugin Plugin> {
        plugins.find(|plugin| plugin.name() == name)
    }

}


#[expect(clippy::single_call_fn, reason = "function extracted to locate to better module")]
pub fn register_core_events(events: &Events, core_id: Uuid) -> Result<(), RuntimeError> {
    let mut new_events = HashMap::new();
    new_events.insert(
        "core:init".into(),
        Event::new(
            schema_from_file(include_str!("../../event/init.json"))?,
            core_id,
        ),
    );
    new_events.insert(
        "core:event".into(),
        Event::new(
            schema_from_file(include_str!("../../event/event.json"))?,
            core_id,
        ),
    );
    new_events.insert(
        "core:endpoint".into(),
        Event::new(
            schema_from_file(include_str!("../../event/endpoint.json"))?,
            core_id,
        ),
    );
    new_events.insert(
        "core:power".into(),
        Event::new(
            schema_from_file(include_str!("../../event/power.json"))?,
            core_id,
        ),
    );
    events.rcu(|map| HashMap::clone(map).union(new_events.clone()));
    Ok(())
}

#[trait_fn(EventHandlerRegisterService for EventHandlerRegister)]
pub(super) fn register<T: AsRef<str>>(
    handler: EventHandlerFuncUnsafeFP,
    plugin_id: Uuid,
    event_name: T,
) -> Result<EventHandler, ServiceError> {
    let event_handler = EventHandler::new_unsafe(handler, Uuid::new_v4());
    let stored_handler = StoredEventHandler::new(event_handler, plugin_id);

    get_gov()
        .error(CoreInternalError)?
        .events()
        .rcu_alter(event_name.as_ref(), |event| {
            event
                .handlers
                .insert(stored_handler.clone())
                .or_error(ServiceError::Duplicate)
        })?;

    Ok(event_handler)
}

#[trait_fn(EventHandlerUnregisterService for EventHandlerUnregister)]
pub(super) fn unregister<S: AsRef<str>>(
    handler_id: Uuid,
    plugin_id: Uuid,
    event_name: S,
) -> Result<(), ServiceError> {
    get_gov()
        .error(CoreInternalError)?
        .events()
        .rcu_alter(event_name.as_ref(), |event| {
            let handler = event
                .handlers
                .iter()
                .find(|stored_handler| stored_handler.handler.id() == handler_id)
                .ok_or(ServiceError::NotFound)?;
            if handler.plugin_id != plugin_id {
                return Err(ServiceError::Unauthorized);
            }
            event.handlers.remove(&handler.clone());
            Ok(())
        })?;

    Ok(())
}

#[trait_fn(EventRegisterService for EventRegister)]
pub(super) fn register<S: AsRef<str>, T: AsRef<str>>(
    event_schema: S,
    plugin_id: Uuid,
    event_name: T,
) -> Result<(), ServiceError> {
    if event_name.as_ref().contains(':') {
        return Err(ServiceError::InvalidString);
    }
    let argument_schema_json = serde_json::from_str(event_schema.as_ref()).error(ServiceError::InvalidJson)?;

    let argument_validator =
        jsonschema::validator_for(&argument_schema_json).error(ServiceError::InvalidSchema)?;
    let event = Event::new(argument_validator, plugin_id);
    let full_name = {
        let gov = get_gov().error(CoreInternalError)?;
        let plugins = gov.loader().plugins().load();
        let plugin_name = plugins.get(&plugin_id).map(Plugin::name).error(ServiceError::NotFound)?;
        if gov.events().load().contains_key(event_name.as_ref()) {
            return Err(ServiceError::Duplicate);
        }
        let full_name = format!("{}:{}", plugin_name, event_name.as_ref());
        gov.events()
            .rcu(|map| map.update(full_name.clone().into(), event.clone()));
        full_name
    };

    let core_id = get_gov().error(CoreInternalError)?.runtime().core_id();

    EventTrigger::trigger(
        core_id,
        "core:event",
        json!({
            "event_name": full_name,
            "argument_schema": argument_schema_json
        })
        .to_string(),
    )
}

#[trait_fn(EventUnregisterService for EventUnregister)]
pub(super) fn unregister<S: AsRef<str>>(
    plugin_id: Uuid,
    event_name: S,
) -> Result<(), ServiceError> {
    {
        let gov = get_gov().error(CoreInternalError)?;
        let events_guard = gov.events().load();
        let event = events_guard
            .get(event_name.as_ref())
            .error(ServiceError::NotFound)?;
        if event.plugin_id != plugin_id {
            return Err(ServiceError::Unauthorized);
        }

        gov.events()
            .rcu(|events| events.without(event_name.as_ref()));
    }

    Ok(())
}

#[trait_fn(EventTriggerService for EventTrigger)]
pub(super) fn trigger<S: AsRef<str>, T: AsRef<str>>(
    plugin_id: Uuid,
    event_name: S,
    args: T,
) -> Result<(), ServiceError> {
    match get_gov().error(CoreInternalError)?.runtime().check_power() {
        PowerState::Shutdown | PowerState::Restart => return Err(ServiceError::ShutingDown),
        PowerState::Running | PowerState::Cancel => {}
    }

    let event_arguments_json = serde_json::from_str(args.as_ref()).error(ServiceError::InvalidJson)?;
    let funcs = {
        let gov = get_gov().error(CoreInternalError)?;
        let events = gov.events().load();
        let event = events
            .get(event_name.as_ref())
            .error(ServiceError::NotFound)?;
        if event.plugin_id != plugin_id {
            return Err(ServiceError::Unauthorized);
        }

        event
            .argument_validator
            .validate(&event_arguments_json)
            .error(ServiceError::InvalidApi)?;
        if event_name.as_ref() == "core:init" {
            let Some(funcs) = sort_handlers(event.handlers.iter(), &gov.loader().plugins().load())
            else {
                return Err(ServiceError::CoreInternalError);
            };
            funcs
        } else {
            event.handlers.iter().map(|stored_handler| stored_handler.handler).collect()
        }
    };
    let executor = get_gov().error(CoreInternalError)?.runtime().event_pool.clone();
    let owned_args = args.as_ref().to_owned();
    executor.execute(move || {
        for func in funcs {
            let _err = func.handle(ContextSupplierImpl, owned_args.clone()).error(ServiceError::PluginInternalError);
        }
    });
    Ok(())
}

#[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
fn sort_handlers<'handler>(
    handlers: impl Iterator<Item = &'handler StoredEventHandler>,
    stored_plugins: &HashMap<Uuid, Plugin>,
) -> Option<Vec<EventHandler>> {
    let nodes: Vec<_> = handlers
        .map(|handler| {
            TopoNode::create_dependency_entry(handler, || stored_plugins.values(), stored_plugins)
        })
        .collect::<Option<_>>()?;
    let mut sorter = TopoSort::new();
    for (node, deps) in nodes {
        sorter.insert_from_set(node, deps);
    }
    
        sorter
            .iter()
            .map(|node| node.ok()?.0.1)
            .collect::<Option<_>>()
}


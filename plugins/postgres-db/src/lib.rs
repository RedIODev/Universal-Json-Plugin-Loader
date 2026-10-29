use std::{borrow::Cow, sync::Arc};

use arc_swap::ArcSwapOption;
use plugin_loader_api::{API_VERSION, ApplicationContext, ErrorMapper as _, PluginInfo, ServiceError, pointer_traits::{EventHandlerFunc, RequestHandlerFunc, plugin_main, trait_fn}};
use serde::Deserialize;
use uuid::Uuid;

static UUID: ArcSwapOption<Uuid> = ArcSwapOption::const_empty();

#[plugin_main]
fn main(uuid: Uuid) -> PluginInfo {
    UUID.store(Some(Arc::new(uuid)));
    PluginInfo::new::<Init, _, _, _>("Postgres DB Plugin", "0.0.1", [], API_VERSION)
}

#[trait_fn(EventHandlerFunc for Init)]
fn handle<'args, F, S>(
    context: F,
    _: S,
) -> Result<(), ServiceError> where F: Fn() -> Result<ApplicationContext, ServiceError>, S: Into<Cow<'args, str>> {
    let uuid = **UUID.load().as_ref().error(ServiceError::PluginInternalError)?;

    context()?.endpoint_request("core:config", uuid, ConfigArgs {})?;
    context()?.register_endpoint::<_,_,_, DataEndpoint>(
        include_str!("endpoint/data-args.json"),
        include_str!("endpoint/data-resp.json"), 
        uuid, "data")?;
    Ok(())
}

#[derive(Deserialize)]

enum DataCommand {
    Store { value:String },
    Load,
    Delete,
}

#[derive(Deserialize)]
struct DataArgs {
    command: DataCommand,
    table: String,
    key: String
}

#[trait_fn(RequestHandlerFunc for DataEndpoint)]
pub fn handle<'args, F: Fn() -> Result<ApplicationContext, ServiceError>, S: Into<Cow<'args, str>>, T: AsRef<str>>(
    _: F,
    plugin_name: T,
    args: S,
) -> Result<String, ServiceError> {
    let args = serde_json::from_str::<DataArgs>(&args.into()).error(ServiceError::InvalidJson)?;

    todo!()
}
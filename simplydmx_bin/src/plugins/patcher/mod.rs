mod services;
mod patcher_context;
mod types;

use async_std::sync::{
	Arc,
	RwLock,
};
use services::GetBaseLayer;

use simplydmx_plugin_framework::*;

use self::patcher_context::PatcherContext;

pub async fn initialize(plugin_context: PluginContext) {
	let patcher_ctx = Arc::new(RwLock::new(PatcherContext::new()));
	plugin_context.register_service(true, GetBaseLayer::new(plugin_context.clone(), Arc::clone(&patcher_ctx))).await.unwrap();
	plugin_context.declare_event::<()>(String::from("patcher.patch_updated")).await.unwrap();
}

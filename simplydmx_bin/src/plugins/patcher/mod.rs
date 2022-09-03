mod services;
mod state;
mod fixture_types;

use async_std::sync::{
	Arc,
	RwLock,
};
use services::GetBaseLayer;

use simplydmx_plugin_framework::*;

use self::state::PatcherContext;

pub async fn initialize(plugin_context: PluginContext) {
	let patcher_ctx = Arc::new(RwLock::new(PatcherContext::new()));

	plugin_context.register_service(true, GetBaseLayer::new(plugin_context.clone(), Arc::clone(&patcher_ctx))).await.unwrap();

	plugin_context.declare_event::<()>(
		"patcher.patch_updated".to_owned(),
		Some("Event emitted by the patcher when a fixture is updated, intended for use by the mixer to trigger a re-blend of the entire show.".to_owned()),
	).await.unwrap();
}

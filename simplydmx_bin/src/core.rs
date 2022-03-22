use std::sync::Arc;

use simplydmx_plugin_framework::{
	plugin::{
		PluginManager,
		PluginContext,
	},
	interpolate_service,
};

#[interpolate_service(
	PluginManager,
	"shutdown",
	"Quit SimplyDMX",
	"Shut down SimplyDMX and all of its components",
)]
impl ShutdownService {
	#[service_main()]
	pub async fn shutdown(self) {
		self.0.shutdown().await;
	}
}

pub async fn initialize(plugin_manager: PluginManager, plugin_context: PluginContext) {
	plugin_context.register_service(true, ShutdownService(Arc::new(plugin_manager.clone()))).await.unwrap();

	let plugin_context_dup = plugin_context.clone();
	plugin_context.spawn(async move {
		async_std::task::sleep(std::time::Duration::from_secs(10)).await;
		let shutdown_service = plugin_context_dup.get_service("core", "shutdown").await.unwrap();
		shutdown_service.call(Vec::new()).await.unwrap();
	}).await.unwrap();
}

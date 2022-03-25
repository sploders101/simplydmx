mod core;

use simplydmx_plugin_framework::{
	PluginManager,
};

#[async_std::main]
async fn main() {

	// Create plugin manager
	let (plugin_manager, shutdown_receiver) = PluginManager::new();

	// Register core plugin
	core::initialize(plugin_manager.clone(), plugin_manager.register_plugin("core", "SimplyDMX Core").await.unwrap()).await;

	// Register other plugins

	// Wait for shutdown request
	shutdown_receiver.recv().await.unwrap();

	// Finish shutdown
	plugin_manager.finish_shutdown().await;

}

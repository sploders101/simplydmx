pub mod plugins;
pub mod type_extensions;

use simplydmx_plugin_framework::{
	PluginManager,
};

#[async_std::main]
async fn main() {

	// Create plugin manager
	let (plugin_manager, shutdown_receiver) = PluginManager::new();

	// Register core plugin
	plugins::core::initialize(
		plugin_manager.clone(),
		plugin_manager.register_plugin(
			"core",
			"SimplyDMX Core",
		).await.unwrap()
	).await;

	plugins::mixer::initialize_mixer(
		plugin_manager.register_plugin(
			"patcher",
			"SimplyDMX Fixture Patcher",
		).await.unwrap(),
	).await;

	// Register other plugins
	plugins::mixer::initialize_mixer(
		plugin_manager.register_plugin(
			"mixer",
			"SimplyDMX Mixer",
		).await.unwrap(),
	).await;

	// #[cfg(target = "output-dmx-e131")]
	// plugins::output_dmx_e131::initialize(
	// 	plugin_manager.register_plugin(
	// 		"output-dmx-e131",
	// 		"E1.31/sACN DMX Output",
	// 	).await.unwrap()
	// ).await;

	// Wait for shutdown request
	shutdown_receiver.recv().await.unwrap();

	// Finish shutdown
	plugin_manager.finish_shutdown().await;

}

#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

pub mod plugins;
pub mod api_utilities;
pub mod utilities;

use simplydmx_plugin_framework::{
	PluginManager,
};

fn main() {

	// Create plugin manager
	let (plugin_manager, shutdown_receiver) = PluginManager::new();

	#[cfg(feature = "gui")]
	{
		// Call tauri and block main thread due to MacOS GUI limitation
		plugins::gui::initialize(
			plugin_manager,
			shutdown_receiver,
		);
	}

	#[cfg(not(feature = "gui"))]
	{
		task::block_on(async_main(plugin_manager, shutdown_receiver));

		// Wait for shutdown request
		shutdown_receiver.recv().await.unwrap();

		// Finish shutdown
		plugin_manager.finish_shutdown().await;

		#[cfg(feature = "verbose-debugging")]
		println!("Successfully shut down.");
	}

}

// Public so the GUI plugin can run it
pub async fn async_main(plugin_manager: PluginManager) {

	// Register core plugin
	plugins::core::initialize(
		plugin_manager.clone(),
		plugin_manager.register_plugin(
			"core",
			"SimplyDMX Core",
		).await.unwrap()
	).await;

	#[cfg(feature = "api")]
	plugins::stdio_api::initialize(
		plugin_manager.register_plugin(
			"api",
			"API Server",
		).await.unwrap(),
	).await;

	let patcher_interface = plugins::patcher::initialize(
		plugin_manager.register_plugin(
			"patcher",
			"SimplyDMX Fixture Patcher",
		).await.unwrap(),
	).await;

	plugins::mixer::initialize_mixer(
		plugin_manager.register_plugin(
			"mixer",
			"SimplyDMX Mixer",
		).await.unwrap(),
		patcher_interface.clone(),
	).await;

	#[cfg(feature = "output-dmx")]
	let dmx_interface = plugins::output_dmx::initialize(
		plugin_manager.register_plugin(
			"output-dmx",
			"E1.31/sACN DMX Output",
		).await.unwrap(),
		patcher_interface.clone(),
	).await;

	#[cfg(feature = "output-dmx-e131")]
	plugins::output_dmx_e131::initialize(
		plugin_manager.register_plugin(
			"output-dmx-e131",
			"E1.31/sACN DMX Output",
		).await.unwrap(),
		dmx_interface.clone(),
	).await;

}

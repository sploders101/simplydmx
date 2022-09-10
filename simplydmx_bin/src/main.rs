#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

pub mod plugins;
pub mod api_utilities;
pub mod utilities;

use async_std::task;
use simplydmx_plugin_framework::{
	PluginManager,
};

fn main() {

	// Create plugin manager
	let plugin_manager = PluginManager::new();

	#[cfg(feature = "gui")]
	{
		// Call tauri and block main thread due to MacOS GUI limitation
		task::block_on(plugins::gui::initialize(plugin_manager));
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

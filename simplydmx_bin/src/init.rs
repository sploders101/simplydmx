use crate::plugins::{
	self,
	saver::SaverInitializationStatus,
};
use simplydmx_plugin_framework::PluginManager;

// Public so the GUI plugin can run it
pub async fn async_main(plugin_manager: PluginManager, data: Option<Vec<u8>>) {

	let saver = plugins::saver::initialize(
		plugin_manager.register_plugin(
			"saver",
			"Data Saver/Loader",
		).await.unwrap(),
		data,
	).await.unwrap();

	// Register core plugin
	plugins::core::initialize(
		plugin_manager.register_plugin(
			"core",
			"SimplyDMX Core",
		).await.unwrap(),
	).await;

	let patcher_interface = plugins::patcher::initialize(
		plugin_manager.register_plugin(
			"patcher",
			"SimplyDMX Fixture Patcher",
		).await.unwrap(),
		saver.clone(),
	).await;

	plugins::mixer::initialize_mixer(
		plugin_manager.register_plugin(
			"mixer",
			"SimplyDMX Mixer",
		).await.unwrap(),
		saver.clone(),
		patcher_interface.clone(),
	).await;

	#[cfg(feature = "output-dmx")]
	let dmx_interface = plugins::output_dmx::initialize(
		plugin_manager.register_plugin(
			"output-dmx",
			"E1.31/sACN DMX Output",
		).await.unwrap(),
		saver.clone(),
		patcher_interface.clone(),
	).await;

	#[cfg(feature = "output-dmx-e131")]
	plugins::output_dmx_e131::initialize(
		plugin_manager.register_plugin(
			"output-dmx-e131",
			"E1.31/sACN DMX Output",
		).await.unwrap(),
		saver.clone(),
		dmx_interface.clone(),
	).await;

	let init_status = saver.finish_initialization().await;
	if let SaverInitializationStatus::FinishedUnsafe = init_status {
		panic!("Save file contains features that are not compatible with this version of SimplyDMX");
	}

}

pub mod interface;
pub mod dmxsource_controller;
pub mod state;

use simplydmx_plugin_framework::PluginContext;
use interface::E131DMXDriver;

use super::{
	output_dmx::interface::DMXInterface,
	saver::SaverInterface,
};

use simplydmx_plugin_framework::*;

pub async fn initialize(plugin_context: PluginContext, saver: SaverInterface, dmx_interface: DMXInterface) -> Result<E131DMXDriver, E131InitializationError> {
	// Create E131 context
	let interface = if let Ok(data) = saver.load_data(&"output_dmx_e131".into()).await {
		if let Some(data) = data {
			E131DMXDriver::from_file(plugin_context, data).await
		} else {
			E131DMXDriver::new(plugin_context).await
		}
	} else {
		return Err(E131InitializationError::UnrecognizedData);
	};

	dmx_interface.register_dmx_driver(interface.clone()).await;

	saver.register_savable("output_dmx_e131", interface.clone()).await.unwrap();

	return Ok(interface);
}

#[portable]
/// An error that could occur while initializing the E131 plugin
pub enum E131InitializationError {
	UnrecognizedData,
}

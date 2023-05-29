pub mod interface;
pub mod controller;

use super::{output_dmx::interface::DMXInterface, saver::SaverInterface};
use interface::OpenDMXDriver;
use simplydmx_plugin_framework::PluginContext;
use simplydmx_plugin_framework::*;
use thiserror::Error;

pub async fn initialize(
	plugin_context: PluginContext,
	saver: SaverInterface,
	dmx_interface: DMXInterface,
) -> Result<OpenDMXDriver, OpenDMXInitializationError> {
	// Create OpenDMX context
	let interface = if let Ok(data) = saver.load_data(&"output_dmx_enttecopendmx".into()).await {
		if let Some(data) = data {
			OpenDMXDriver::from_file(plugin_context, data).await
		} else {
			OpenDMXDriver::new(plugin_context).await
		}
	} else {
		return Err(OpenDMXInitializationError::UnrecognizedData);
	};

	dmx_interface.register_dmx_driver(interface.clone()).await;

	saver
		.register_savable("output_dmx_enttecopendmx", interface.clone())
		.await
		.unwrap();

	return Ok(interface);
}

#[portable]
#[derive(Error)]
/// An error that could occur while initializing the E131 plugin
pub enum OpenDMXInitializationError {
	#[error("The show file could not be parsed.")]
	UnrecognizedData,
}

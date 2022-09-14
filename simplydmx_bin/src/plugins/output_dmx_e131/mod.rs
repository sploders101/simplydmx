pub mod interface;
pub mod dmxsource_controller;
pub mod state;

use simplydmx_plugin_framework::PluginContext;
use interface::E131DMXDriver;

use super::{
	output_dmx::interface::DMXInterface,
	saver::SaverInterface,
};

pub async fn initialize(plugin_context: PluginContext, saver: SaverInterface, dmx_interface: DMXInterface) -> E131DMXDriver {
	let interface = E131DMXDriver::new(plugin_context).await;

	dmx_interface.register_dmx_driver(interface.clone()).await;

	return interface;
}

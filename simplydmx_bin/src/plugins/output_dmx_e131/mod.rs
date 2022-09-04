pub mod interface;
pub mod dmxsource_controller;
pub mod state;

use simplydmx_plugin_framework::PluginContext;
use interface::E131DMXDriver;

use super::output_dmx::interface::DMXInterface;

pub async fn initialize(plugin_context: PluginContext, dmx_interface: DMXInterface) -> E131DMXDriver {
	let interface = E131DMXDriver::new(plugin_context);

	dmx_interface.register_output(interface.clone()).await;

	return interface;
}

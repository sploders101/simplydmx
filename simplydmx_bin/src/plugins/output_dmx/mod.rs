pub mod interface;
pub mod driver_types;
pub mod fixture_types;
pub mod state;

use simplydmx_plugin_framework::*;

use self::interface::DMXInterface;

use super::patcher::PatcherInterface;


pub async fn initialize(plugin_context: PluginContext, patcher_interface: PatcherInterface) -> DMXInterface {

	// Create plugin interface
	let output_context = DMXInterface::new(plugin_context);

	patcher_interface.register_output_driver(output_context.clone()).await;

	return output_context;

}

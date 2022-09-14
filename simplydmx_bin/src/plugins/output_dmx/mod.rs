pub mod interface;
pub mod driver_types;
pub mod fixture_types;
pub mod state;
pub mod services;

use simplydmx_plugin_framework::*;

use self::interface::DMXInterface;

use super::{
	patcher::PatcherInterface,
	saver::SaverInterface,
};


pub async fn initialize(plugin_context: PluginContext, saver: SaverInterface, patcher_interface: PatcherInterface) -> DMXInterface {

	// Create plugin interface
	let output_context = DMXInterface::new(plugin_context.clone());

	patcher_interface.register_output_driver(output_context.clone()).await;

	plugin_context.declare_event::<Vec<u8>>(
		"dmx.output".into(),
		Some("The output of the DMX plugin, for display by the UI. This should not be used by DMX drivers.".into()),
	).await.unwrap();

	plugin_context.declare_event::<()>(
		"dmx.universe_removed".into(),
		Some("Emitted when a universe is removed from the DMX plugin".into()),
	).await.unwrap();

	plugin_context.register_service(true, services::CreateUniverse::new(output_context.clone())).await.unwrap();
	plugin_context.register_service(true, services::DeleteUniverse::new(output_context.clone())).await.unwrap();
	plugin_context.register_service(true, services::LinkUniverse::new(output_context.clone())).await.unwrap();
	plugin_context.register_service(true, services::UnlinkUniverse::new(output_context.clone())).await.unwrap();

	return output_context;

}

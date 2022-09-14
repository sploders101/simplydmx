pub mod driver_plugin_api;
mod state;
mod fixture_types;
mod services;

use async_std::sync::{
	Arc,
	RwLock,
};

mod interface;
pub use interface::PatcherInterface;

use simplydmx_plugin_framework::*;

use self::{
	state::PatcherContext,
	services::{
		ImportFixtureDefinition,
		CreateFixture,
	},
};

use super::saver::SaverInterface;

pub async fn initialize(plugin_context: PluginContext, saver: SaverInterface) -> PatcherInterface {
	let patcher_interface = PatcherInterface::new(plugin_context.clone(), Arc::new(RwLock::new(PatcherContext::new())));

	plugin_context.declare_event::<()>(
		"patcher.patch_updated".to_owned(),
		Some("Event emitted by the patcher when a fixture is updated, intended for use by the mixer to trigger a re-blend of the entire show.".to_owned()),
	).await.unwrap();

	plugin_context.declare_event::<()>(
		"patcher.new_fixture".into(),
		Some("Event emitted when a new fixture type has been successfully imported into SimplyDMX".into()),
	).await.unwrap();

	plugin_context.register_service(true, ImportFixtureDefinition::new(patcher_interface.clone())).await.unwrap();
	plugin_context.register_service(true, CreateFixture::new(patcher_interface.clone())).await.unwrap();

	return patcher_interface;
}

pub mod driver_plugin_api;
mod fixture_types;
mod interface;
mod services;
mod state;

use self::{
	services::{
		CreateFixture, EditFixture, EditFixturePlacement, GetCreationForm, GetEditForm,
		GetPatcherState, ImportFixtureDefinition,
	},
	state::{PatcherContext, VisualizationInfo},
};
use super::saver::SaverInterface;
pub use interface::PatcherInterface;
use simplydmx_plugin_framework::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub async fn initialize(plugin_context: PluginContext, saver: SaverInterface) -> Result<PatcherInterface, PatcherInitializationError> {
	// Create patcher context
	let patcher_interface = if let Ok(data) = saver.load_data(&"patcher".into()).await {
		if let Some(data) = data {
			PatcherInterface::new(plugin_context.clone(), Arc::new(RwLock::new(PatcherContext::from_file(data))))
		} else {
			PatcherInterface::new(plugin_context.clone(), Arc::new(RwLock::new(PatcherContext::new())))
		}
	} else {
		return Err(PatcherInitializationError::UnrecognizedData);
	};

	plugin_context.declare_event::<()>(
		"patcher.patch_updated".to_owned(),
		Some("Event emitted by the patcher when a fixture is updated, intended for use by the mixer to trigger a re-blend of the entire show.".to_owned()),
	).await.unwrap();

	plugin_context.declare_event::<()>(
		"patcher.new_fixture".into(),
		Some("Event emitted when a new fixture type has been successfully imported into SimplyDMX".into()),
	).await.unwrap();

	plugin_context.declare_event::<(Uuid, VisualizationInfo)>(
		"patcher.visualization_updated".into(),
		Some("Event emitted when a fixture's visualization properties have been updated".into()),
	).await.unwrap();

	plugin_context.register_service(true, ImportFixtureDefinition::new(patcher_interface.clone())).await.unwrap();
	plugin_context.register_service(true, CreateFixture::new(patcher_interface.clone())).await.unwrap();
	plugin_context.register_service(true, GetCreationForm::new(patcher_interface.clone())).await.unwrap();
	plugin_context.register_service(true, GetPatcherState::new(patcher_interface.clone())).await.unwrap();
	plugin_context.register_service(true, GetEditForm::new(patcher_interface.clone())).await.unwrap();
	plugin_context.register_service(true, EditFixture::new(patcher_interface.clone())).await.unwrap();
	plugin_context.register_service(true, EditFixturePlacement::new(patcher_interface.clone())).await.unwrap();

	saver.register_savable("patcher", patcher_interface.clone()).await.unwrap();

	return Ok(patcher_interface);
}

#[portable]
/// An error that could occur while initializing the patcher plugin
pub enum PatcherInitializationError {
	UnrecognizedData,
}

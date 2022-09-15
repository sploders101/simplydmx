mod commands;
mod state;
mod blender;

/// Exported types may be used by other plugins when communicating with the mixer
pub mod exported_types {
	pub use super::state::{
		Submaster,
		AbstractLayerLight,
		BlenderValue,
		FullMixerOutput,
		FixtureMixerOutput,
		BlendingScheme,
		FullMixerBlendingData,
		SnapData,
		BlendingData,
	};
}

use std::sync::Arc;
use async_std::{
	sync::Mutex,
};
use async_trait::async_trait;
use simplydmx_plugin_framework::*;

use state::{
	MixerContext,
	FullMixerOutput,
};

use self::blender::UpdateList;

use super::{
	patcher::PatcherInterface,
	saver::{
		SaverInterface,
		Savable,
	},
};

pub async fn initialize_mixer(plugin_context: PluginContext, saver: SaverInterface, patcher: PatcherInterface) -> Result<MixerInterface, MixerInitializationError> {

	// Create mixer context
	let mixer_context = Arc::new(Mutex::new(if let Ok(data) = saver.load_data(&"mixer".into()).await {
		if let Some(data) = data {
			MixerContext::from_file(data)
		} else {
			MixerContext::new()
		}
	} else {
		return Err(MixerInitializationError::UnrecognizedData);
	}));


	// Declare events

	plugin_context.declare_event::<FullMixerOutput>(
		"mixer.layer_bin_output".to_owned(),
		Some("Emitted when a layer bin has been updated in the mixer".to_owned()),
	).await.unwrap();

	plugin_context.declare_event::<FullMixerOutput>(
		"mixer.final_output".to_owned(),
		Some("Emitted when the mixer's final output has been updated".to_owned()),
	).await.unwrap();


	// Start blender task
	let update_sender = blender::start_blender(plugin_context.clone(), Arc::clone(&mixer_context), patcher).await;

	// Send kickstart to blender task to recover any data that was saved
	update_sender.send(UpdateList::All).await.unwrap();

	// Register services
	plugin_context.register_service(true, commands::EnterBlindMode::new(plugin_context.clone(), Arc::clone(&mixer_context))).await.unwrap();
	plugin_context.register_service(true, commands::SetBlindOpacity::new(plugin_context.clone(), Arc::clone(&mixer_context), update_sender.clone())).await.unwrap();
	plugin_context.register_service(true, commands::GetBlindOpacity::new(plugin_context.clone(), Arc::clone(&mixer_context))).await.unwrap();
	plugin_context.register_service(true, commands::RevertBlind::new(plugin_context.clone(), Arc::clone(&mixer_context), update_sender.clone())).await.unwrap();
	plugin_context.register_service(true, commands::CommitBlind::new(plugin_context.clone(), Arc::clone(&mixer_context), update_sender.clone())).await.unwrap();
	plugin_context.register_service(true, commands::CreateLayer::new(plugin_context.clone(), Arc::clone(&mixer_context))).await.unwrap();
	plugin_context.register_service(true, commands::SetLayerContents::new(plugin_context.clone(), Arc::clone(&mixer_context), update_sender.clone())).await.unwrap();
	plugin_context.register_service(true, commands::GetLayerContents::new(plugin_context.clone(), Arc::clone(&mixer_context))).await.unwrap();
	plugin_context.register_service(true, commands::SetLayerOpacity::new(plugin_context.clone(), Arc::clone(&mixer_context), update_sender.clone())).await.unwrap();
	plugin_context.register_service(true, commands::GetLayerOpacity::new(plugin_context.clone(), Arc::clone(&mixer_context))).await.unwrap();
	plugin_context.register_service(true, commands::DeleteLayer::new(plugin_context.clone(), Arc::clone(&mixer_context), update_sender.clone())).await.unwrap();

	// Create mixer interface
	let interface = MixerInterface::new(plugin_context, mixer_context);

	// Register saving mechanism
	saver.register_savable("mixer", interface.clone()).await.unwrap();

	return Ok(interface);
}

#[derive(Clone)]
pub struct MixerInterface(PluginContext, Arc<Mutex<MixerContext>>);
impl MixerInterface {
	pub fn new(plugin_context: PluginContext, mixer_context: Arc<Mutex<MixerContext>>) -> Self {
		return Self(plugin_context, mixer_context);
	}
}

#[async_trait]
impl Savable for MixerInterface {
	async fn save_data(&self) -> Result<Option<Vec<u8>>, String> {
		return Ok(Some(self.1.lock().await.serialize_cbor()?));
	}
}

#[portable]
#[derive(Debug)]
pub enum MixerInitializationError {
	UnrecognizedData,
}

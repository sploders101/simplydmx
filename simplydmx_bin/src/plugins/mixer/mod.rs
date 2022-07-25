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
		BlendingScheme,
		FullMixerBlendingData,
	};
}

use std::sync::Arc;
use async_std::{
	sync::Mutex,
};
use simplydmx_plugin_framework::*;

use state::MixerContext;

pub async fn initialize_mixer(plugin_context: PluginContext) {

	// Create mixer context
	let mixer_context = Arc::new(Mutex::new(MixerContext::new()));

	// Start blender task
	let update_sender = blender::start_blender(plugin_context.clone(), Arc::clone(&mixer_context)).await;

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

}

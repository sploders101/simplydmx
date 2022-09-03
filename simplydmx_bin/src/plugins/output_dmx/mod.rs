pub mod services;
pub mod driver_types;
pub mod fixture_types;
pub mod state;

use std::{
	sync::Arc,
};
use async_std::sync::{
	RwLock,
};

use simplydmx_plugin_framework::*;

pub async fn initialize(plugin_context: PluginContext) {

	// Create container for DMX outputs
	let output_context = Arc::new(RwLock::new(state::DMXState::new()));

	// Register services
	plugin_context.register_service(true, services::RegisterOutputType::new(plugin_context.clone(), Arc::clone(&output_context))).await.unwrap();
	plugin_context.register_service(true, services::QueryOutputTypes::new(Arc::clone(&output_context))).await.unwrap();

}

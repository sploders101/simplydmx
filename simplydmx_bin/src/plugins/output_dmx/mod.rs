pub mod services;
pub mod types;

use std::{
	sync::Arc,
	collections::HashMap,
};
use async_std::sync::{
	RwLock,
};

use simplydmx_plugin_framework::*;

pub async fn initialize(plugin_context: PluginContext) {

	// Create container for DMX outputs
	let output_context = Arc::new(types::OutputContext {
		output_types: RwLock::new(HashMap::new()),
	});

	// Register services
	plugin_context.register_service(true, services::RegisterOutputType::new(plugin_context.clone(), Arc::clone(&output_context))).await.unwrap();
	plugin_context.register_service(true, services::QueryOutputTypes::new(Arc::clone(&output_context))).await.unwrap();

}

use std::sync::Arc;
use async_std::sync::RwLock;

use simplydmx_plugin_framework::*;
use super::state::{
	DMXState,
};
use super::driver_types::{
	DMXDriverDescriptor,
	DisplayableDMXDriver,
};


#[interpolate_service(
	"register_output_type",
	"Register Output Type",
	"Registers a new DMX transport type",
)]
impl RegisterOutputType {

	#![inner_raw(PluginContext, Arc::<RwLock::<DMXState>>)]

	pub fn new(plugin_context: PluginContext, output_context: Arc::<RwLock::<DMXState>>) -> RegisterOutputType {
		return RegisterOutputType(plugin_context, output_context);
	}

	#[service_main(
		("Output ID", "The unique ID of the output to be used for referencing this transport. This ID should be static so it can be saved for later."),
		("Output Name", "The name of the output type for display in the GUI"),
		("Output Description", "A brief description of the output type including alternate names, carrier transports (like ethernet), etc"),
		("Plugin ID", "The ID of the plugin that should be used to manage this DMX transport"),
		("'Register Universe' Service ID", "The ID of the service used to create a new universe"),
		("'Delete Universe' Service ID", "The ID of the service used to delete an existing universe"),
		("Output Channel", "The channel ID to use on the event bus when sending fixture updates to the transport plugin"),
	)]
	async fn main(self,
		id: String,
		name: String,
		description: String,
		plugin_id: String,
		register_universe_service: String,
		delete_universe_service: String,
		output_service: String,
	) {
		let mut ctx = self.1.write().await;
		ctx.output_types.insert(String::clone(&id), DMXDriverDescriptor {
			id: String::clone(&id),
			name,
			description,
			plugin_id,
			register_universe_service,
			delete_universe_service,
			output_service,
		});
		drop(ctx);
		self.0.emit("output_dmx.output_registered".into(), FilterCriteria::None, id).await;
	}

}


#[interpolate_service(
	"query_output_types",
	"Query output types",
	"Queries a list of valid output types with presentable metadata for the UI",
)]
impl QueryOutputTypes {

	#![inner_raw(Arc::<RwLock::<DMXState>>)]

	pub fn new(output_context: Arc::<RwLock::<DMXState>>) -> QueryOutputTypes {
		return QueryOutputTypes(output_context);
	}

	#[service_main(
		("Outputs", "List of output types with presentable metadata"),
	)]
	async fn main(self) -> Vec<DisplayableDMXDriver> {
		let ctx = self.0.write().await;
		return ctx.output_types.values().map(|output| DisplayableDMXDriver {
			id: String::clone(&output.id),
			name: String::clone(&output.name),
			description: String::clone(&output.name),
		}).collect();
	}
}

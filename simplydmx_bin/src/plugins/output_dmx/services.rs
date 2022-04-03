use simplydmx_plugin_framework::*;
use super::types::{
	OutputContext,
	OutputDescriptor,
	DisplayableOutput,
};


#[interpolate_service(
	(PluginContext, OutputContext),
	"register_output_type",
	"Register Output Type",
	"Registers a new DMX transport type",
)]
impl RegisterOutputType {
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
		register_universe_id: String,
		delete_universe_id: String,
		output_channel: String,
	) {
		let mut available_outputs = self.0.1.output_types.write().await;
		available_outputs.insert(String::clone(&id), OutputDescriptor {
			id: String::clone(&id),
			name,
			description,
			plugin_id,
			register_universe_id,
			delete_universe_id,
			output_channel,
		});
		drop(available_outputs);
		self.0.0.emit("output_dmx.output_registered".into(), id).await;
	}
}


#[interpolate_service(
	OutputContext,
	"query_output_types",
	"Query output types",
	"Queries a list of valid output types with presentable metadata for the UI",
)]
impl QueryOutputTypes {
	#[service_main(
		("Outputs", "List of output types with presentable metadata"),
	)]
	async fn main(self) -> Vec<DisplayableOutput> {
		let output_types = self.0.output_types.write().await;
		return output_types.values().map(|output| DisplayableOutput {
			id: String::clone(&output.id),
			name: String::clone(&output.name),
			description: String::clone(&output.name),
		}).collect();
	}
}

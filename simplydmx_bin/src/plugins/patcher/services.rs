use std::collections::HashMap;
use async_std::sync::Arc;
use simplydmx_plugin_framework::*;

use crate::plugins::mixer::exported_types::FullMixerOutput;

#[interpolate_service(
	"get_base_layer",
	"Query output types",
	"Queries a list of valid output types with presentable metadata for the UI",
)]
impl GetBaseLayer {

	#![inner(())]

	pub fn new() -> Self {
		return GetBaseLayer(Arc::new(()));
	}

	#[service_main(
		("Outputs", "List of output types with presentable metadata"),
	)]
	async fn main(self) -> FullMixerOutput {
		return HashMap::new();
	}
}

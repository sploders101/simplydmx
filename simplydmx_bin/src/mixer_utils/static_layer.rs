use std::collections::HashMap;

use async_trait::async_trait;

use simplydmx_plugin_framework::*;

use super::{
	layer::MixerLayer,
	state::{
		SubmasterData,
		FullMixerOutput,
	},
	data_sources::LayerDataSourcesLocked,
	default_blender::blend_layer,
};

#[portable]
pub struct StaticLayer {
	pub values: SubmasterData,
}

impl Default for StaticLayer {
	fn default() -> Self {
		return StaticLayer {
			values: HashMap::new(),
		}
	}
}

#[async_trait]
impl MixerLayer for StaticLayer {
	fn animated(&self) -> bool { false }
	async fn blend(&self, cumulative_layer: &mut FullMixerOutput, data_sources: &LayerDataSourcesLocked, opacity: u16) {
		blend_layer(cumulative_layer, data_sources, opacity, &self.values).await;
	}
}

use rustc_hash::FxHashMap;

use async_trait::async_trait;

use crate::mixer_utils::state::FullMixerBlendingData;
use simplydmx_plugin_framework::*;
use uuid::Uuid;

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
/// Defines a static submaster
pub struct StaticLayer {
	pub name: String,
	pub values: SubmasterData,
}
impl StaticLayer {
	pub fn new(name: String) -> StaticLayer {
		return StaticLayer {
			name,
			values: FxHashMap::default(),
		};
	}
}

#[async_trait]
impl MixerLayer for StaticLayer {
	fn animated(&self) -> bool { false }
	async fn cleanup(&mut self, patcher_data: &(FullMixerOutput, FullMixerBlendingData)) {
		// Iterate over fixtures
		let fixture_keys: Vec<Uuid> = self.values.keys().cloned().collect();
		for fixture_id in fixture_keys {
			if let Some(fixture_base) = patcher_data.0.get(&fixture_id) {
				let fixture_data = self.values.get_mut(&fixture_id).unwrap(); // unwrapped because key was sourced from here
				// Iterate over attributes
				let attribute_keys: Vec<String> = fixture_data.keys().cloned().collect();
				for attribute_id in attribute_keys {
					if !fixture_base.contains_key(&attribute_id) {
						// Delete attributes that no longer exist
						fixture_data.remove(&attribute_id);
					}
				}
			} else {
				// Delete fixtures that no longer exist
				self.values.remove(&fixture_id);
			}
		}
	}
	async fn blend(&self, cumulative_layer: &mut FullMixerOutput, data_sources: &LayerDataSourcesLocked, opacity: u16) {
		blend_layer(cumulative_layer, data_sources, opacity, &self.values);
	}
}

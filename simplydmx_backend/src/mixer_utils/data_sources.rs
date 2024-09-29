use super::state::{BlendingData, FullMixerBlendingData, FullMixerOutput, LightAttributeId};
use crate::utils::id_alloc::LargeIdRaw;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct LayerDataSources {
	pub base_layer: RwLock<Arc<FullMixerOutput>>,
	pub blending_data: RwLock<Arc<FullMixerBlendingData>>,
}

impl LayerDataSources {
	pub async fn lock(&self) -> LayerDataSourcesLocked {
		return LayerDataSourcesLocked {
			base_layer: Arc::clone(&*self.base_layer.read().await),
			blending_data: Arc::clone(&*self.blending_data.read().await),
		};
	}
}

pub struct LayerDataSourcesLocked {
	base_layer: Arc<FullMixerOutput>,
	blending_data: Arc<FullMixerBlendingData>,
}

impl LayerDataSourcesLocked {
	pub fn base_layer<'a>(&'a self) -> &'a FullMixerOutput {
		&self.base_layer
	}
	pub fn blending_data<'a>(
		&'a self,
		fixture_id: LargeIdRaw,
	) -> Option<&'a FxHashMap<LightAttributeId, BlendingData>> {
		self.blending_data.get(&fixture_id)
	}
}

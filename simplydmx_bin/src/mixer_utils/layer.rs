use std::any::Any;

use async_trait::async_trait;

use super::{
	state::FullMixerOutput,
	data_sources::LayerDataSourcesLocked,
};

#[async_trait]
pub trait MixerLayer: Any + Clone + 'static {
	fn animated(&self) -> bool;
	async fn blend(&self, cumulative_layer: &mut FullMixerOutput, data_sources: &LayerDataSourcesLocked, opacity: u16);
}

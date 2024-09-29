use std::any::Any;

use crate::mixer_utils::state::FullMixerBlendingData;
use async_trait::async_trait;

use super::{data_sources::LayerDataSourcesLocked, state::FullMixerOutput};

#[async_trait]
pub trait MixerLayer: Any + Clone + 'static {
	fn animated(&self) -> bool;
	async fn cleanup(&mut self, patcher_data: &(FullMixerOutput, FullMixerBlendingData));
	async fn blend(
		&self,
		cumulative_layer: &mut FullMixerOutput,
		data_sources: &LayerDataSourcesLocked,
		opacity: u16,
	);
}

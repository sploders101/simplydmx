use std::collections::HashMap;
use async_std::sync::{
	Arc,
	RwLock,
};
use simplydmx_plugin_framework::*;

use crate::plugins::mixer::exported_types::{
	FullMixerOutput,
	FullMixerBlendingData,
	BlendingData,
	SnapData,
};

use super::{
	state::PatcherContext,
	fixture_types::{
		ChannelType,
		ChannelSize,
		Segment,
	},
};

#[derive(Clone)]
pub struct PatcherInterface(PluginContext, Arc::<RwLock::<PatcherContext>>);
impl PatcherInterface {

	pub fn new(plugin_context: PluginContext, patcher_ctx: Arc::<RwLock::<PatcherContext>>) -> Self {
		return PatcherInterface(plugin_context, patcher_ctx);
	}

	pub async fn main(&self) -> (FullMixerOutput, FullMixerBlendingData) {
		let mut default_values: FullMixerOutput = HashMap::new();
		let mut blending_data: FullMixerBlendingData = HashMap::new();

		let ctx = self.1.read().await;

		for (fixture_id, fixture_data) in ctx.fixtures.iter() {
			if let Some(fixture_info) = ctx.library.get(fixture_id) {
				if let Some(fixture_personality) = fixture_info.personalities.get(&fixture_data.personality) {
					// Create containers for this fixture
					let mut fixture_defaults = HashMap::new();
					let mut fixture_blending_data = HashMap::new();

					// Iterate through channels, populating the fixture containers
					for channel_id in fixture_personality.available_channels.iter() {
						if let Some(channel_info) = fixture_info.channels.get(channel_id) {
							match &channel_info.ch_type {
								ChannelType::Linear { priority } => {
									// Insert default value
									fixture_defaults.insert(channel_id.clone(), channel_info.default);
									// Insert blending instructions
									fixture_blending_data.insert(channel_id.clone(), BlendingData {
										scheme: priority.clone(),
										snap: SnapData::NoSnap,
										allow_wrap: false,
										max_value: get_max_value(&channel_info.size),
										min_value: 0,
									});
								},
								ChannelType::Segmented { segments, priority, snapping } => {
									// Insert default value
									fixture_defaults.insert(channel_id.clone(), channel_info.default);
									// Insert blending instructions
									fixture_blending_data.insert(channel_id.clone(), BlendingData {
										scheme: priority.clone(),
										snap: snapping.clone().unwrap_or(SnapData::NoSnap),
										allow_wrap: false,
										max_value: get_max_value_segments(&segments),
										min_value: get_min_value_segments(&segments),
									});
								},
							}
						} else {
							log_error!(self.0, "Could not find channel {} in fixture {}", channel_id, &fixture_info.name);
						}
					}

					// Insert fixture values into show containers
					default_values.insert(fixture_id.clone(), fixture_defaults);
					blending_data.insert(fixture_id.clone(), fixture_blending_data);
				} else {
					log_error!(self.0, "Could not find personality {} for fixture {}", &fixture_data.personality, &fixture_info.name);
				}
			} else {
				log_error!(self.0, "Could not find fixture {}", fixture_id);
			}
		}

		return (default_values, blending_data);
	}

}

fn get_max_value(channel_size: &ChannelSize) -> u16 {
	return match channel_size {
		ChannelSize::U8 => 255,
		ChannelSize::U16 => 65535,
	}
}

fn get_max_value_segments(segments: &[Segment]) -> u16 {
	let mut max_value: Option<u16> = None;
	for segment in segments {
		if let Some(current_max) = max_value {
			if current_max > segment.end {
				max_value = Some(segment.end);
			}
		} else {
			max_value = Some(segment.end);
		}
	}

	return max_value.unwrap_or(0);
}

fn get_min_value_segments(segments: &[Segment]) -> u16 {
	let mut min_value: Option<u16> = None;
	for segment in segments {
		if let Some(current_min) = min_value {
			if current_min < segment.start {
				min_value = Some(segment.start);
			}
		} else {
			min_value = Some(segment.start);
		}
	}

	return min_value.unwrap_or(0);
}

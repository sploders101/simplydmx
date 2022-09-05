use std::collections::HashMap;
use async_std::sync::{
	Arc,
	RwLock,
};
use futures::{
	FutureExt,
	future::join_all,
};
use simplydmx_plugin_framework::*;
use uuid::Uuid;

use crate::{
	plugins::mixer::exported_types::{
		FullMixerOutput,
		FullMixerBlendingData,
		BlendingData,
		SnapData,
	},
	utilities::serialized_data::SerializedData,
};

use super::{
	state::PatcherContext,
	fixture_types::{
		ChannelType,
		ChannelSize,
		Segment,
	},
	driver_plugin_api::{
		self,
		OutputDriver,
		SharableStateWrapper,
		FixtureBundle, FixtureInstance,
	},
};

#[derive(Clone)]
pub struct PatcherInterface(PluginContext, Arc::<RwLock::<PatcherContext>>);
impl PatcherInterface {

	pub fn new(plugin_context: PluginContext, patcher_ctx: Arc::<RwLock::<PatcherContext>>) -> Self {
		return PatcherInterface(plugin_context, patcher_ctx);
	}

	/// Gets the initial background layer for the mixer to blend data with.
	pub async fn get_base_layer(&self) -> (FullMixerOutput, FullMixerBlendingData) {
		let mut default_values: FullMixerOutput = HashMap::new();
		let mut blending_data: FullMixerBlendingData = HashMap::new();

		let ctx = self.1.read().await;

		for (fixture_id, fixture_data) in ctx.sharable.fixtures.iter() {
			if let Some(fixture_info) = ctx.sharable.library.get(fixture_id) {
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

	/// Registers an output plugin for use by the patcher.
	pub async fn register_output_driver<T: OutputDriver>(&self, plugin: T) {
		let mut ctx = self.1.write().await;
		ctx.output_drivers.insert(plugin.get_id(), Arc::new(Box::new(plugin)));
	}

	/// Import a fixture bundle
	pub async fn import_fixture(&self, fixture_bundle: FixtureBundle) -> Result<(), ImportFixtureError> {
		let mut ctx = self.1.write().await;
		if let Some(output_driver) = ctx.output_drivers.get(&fixture_bundle.fixture_info.output_driver) {
			if let Err(controller_error) = output_driver.import_fixture(&fixture_bundle.fixture_info.id, fixture_bundle.output_info).await {
				return Err(ImportFixtureError::ErrorFromController(controller_error));
			} else {
				// Controller successfully loaded protocol-specific details
				ctx.sharable.library.insert(fixture_bundle.fixture_info.id.clone(), fixture_bundle.fixture_info);
				self.0.emit("patcher.new_fixture".into(), FilterCriteria::None, ()).await;
				return Ok(());
			}
		} else {
			return Err(ImportFixtureError::UnknownController);
		}
	}

	/// Create a fixture
	pub async fn create_fixture(&self, fixture_type: Uuid, personality: String, name: Option<String>, comments: Option<String>, form_data: SerializedData) -> Result<Uuid, CreateFixtureError> {
		let mut ctx = self.1.write().await;
		if let Some(fixture_type_info) = ctx.sharable.library.get(&fixture_type) {
			if let Some(controller) = ctx.output_drivers.get(&fixture_type_info.output_driver) {
				let instance_uuid = Uuid::new_v4();
				if let Err(controller_error) = controller.create_fixture_instance(&instance_uuid, form_data).await {
					return Err(CreateFixtureError::ErrorFromController(controller_error));
				} else {
					// Controller successfully loaded protocol-specific details
					ctx.sharable.fixtures.insert(instance_uuid.clone(), FixtureInstance {
						id: instance_uuid.clone(),
						fixture_id: fixture_type,
						personality,
						name,
						comments,
					});
					return Ok(instance_uuid);
				}
			} else {
				return Err(CreateFixtureError::ControllerMissing);
			}
		} else {
			return Err(CreateFixtureError::FixtureTypeMissing);
		}
	}

	/// Write values to the output plugins
	pub async fn write_values(&self, data: Arc<FullMixerOutput>) {
		let ctx = self.1.read().await;

		let mut futures = Vec::new();
		let data = Arc::new(data);
		for driver in ctx.output_drivers.values() {
			futures.push(driver.send_updates(PatcherInterface::clone(self), Arc::clone(&data)).fuse());
		}
		join_all(futures).await;
	}

	pub async fn get_sharable_state<'a>(&'a self) -> SharableStateWrapper<'a> {
		let ctx = self.1.read().await;
		return SharableStateWrapper::new(ctx);
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

#[portable]
pub enum ImportFixtureError {
	UnknownController,
	ErrorFromController(driver_plugin_api::ImportError),
}

#[portable]
pub enum CreateFixtureError {
	FixtureTypeMissing,
	ControllerMissing,
	ErrorFromController(driver_plugin_api::CreateInstanceError),
}

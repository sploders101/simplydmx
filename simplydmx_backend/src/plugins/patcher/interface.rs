use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use futures::{future::join_all, FutureExt};
use simplydmx_plugin_framework::*;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

use crate::{
	impl_anyhow,
	mixer_utils::state::{BlendingData, FullMixerBlendingData, FullMixerOutput, SnapData},
	plugins::saver::Savable,
	utilities::{forms::FormDescriptor, serialized_data::SerializedData},
};

use super::{
	driver_plugin_api::{self, FixtureBundle, FixtureInstance, OutputDriver, SharableStateWrapper},
	fixture_types::{ChannelSize, ChannelType, Segment},
	state::PatcherContext,
};

#[derive(Clone)]
pub struct PatcherInterface(PluginContext, Arc<RwLock<PatcherContext>>);
impl PatcherInterface {
	pub fn new(plugin_context: PluginContext, patcher_ctx: Arc<RwLock<PatcherContext>>) -> Self {
		return PatcherInterface(plugin_context, patcher_ctx);
	}

	/// Gets the initial background layer for the mixer to blend data with.
	pub async fn get_base_layer(&self) -> (FullMixerOutput, FullMixerBlendingData) {
		let mut default_values: FullMixerOutput = HashMap::new();
		let mut blending_data: FullMixerBlendingData = HashMap::new();

		let ctx = self.1.read().await;

		for (fixture_id, fixture_data) in ctx.sharable.fixtures.iter() {
			if let Some(fixture_info) = ctx.sharable.library.get(&fixture_data.fixture_id) {
				if let Some(fixture_personality) =
					fixture_info.personalities.get(&fixture_data.personality)
				{
					// Create containers for this fixture
					let mut fixture_defaults = HashMap::new();
					let mut fixture_blending_data = HashMap::new();

					// Iterate through channels, populating the fixture containers
					for channel_id in fixture_personality.available_channels.iter() {
						if let Some(channel_info) = fixture_info.channels.get(channel_id) {
							match &channel_info.ch_type {
								ChannelType::Linear { priority } => {
									// Insert default value
									fixture_defaults
										.insert(channel_id.clone(), channel_info.default);
									// Insert blending instructions
									fixture_blending_data.insert(
										channel_id.clone(),
										BlendingData {
											scheme: priority.clone(),
											snap: SnapData::NoSnap,
											allow_wrap: false,
											max_value: get_max_value(&channel_info.size),
											min_value: 0,
										},
									);
								}
								ChannelType::Segmented {
									segments,
									priority,
									snapping,
								} => {
									// Insert default value
									fixture_defaults
										.insert(channel_id.clone(), channel_info.default);
									// Insert blending instructions
									fixture_blending_data.insert(
										channel_id.clone(),
										BlendingData {
											scheme: priority.clone(),
											snap: snapping.clone().unwrap_or(SnapData::NoSnap),
											allow_wrap: false,
											max_value: get_max_value_segments(&segments),
											min_value: get_min_value_segments(&segments),
										},
									);
								}
							}
						} else {
							log_error!(
								self.0,
								"Could not find channel {} in fixture {}",
								channel_id,
								&fixture_info.name
							);
						}
					}

					// Insert fixture values into show containers
					default_values.insert(fixture_id.clone(), fixture_defaults);
					blending_data.insert(fixture_id.clone(), fixture_blending_data);
				} else {
					log_error!(
						self.0,
						"Could not find personality {} for fixture {}",
						&fixture_data.personality,
						&fixture_info.name
					);
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
		ctx.output_drivers
			.insert(plugin.get_id(), Arc::new(Box::new(plugin)));
	}

	/// Import a fixture bundle
	pub async fn import_fixture(
		&self,
		fixture_bundle: FixtureBundle,
	) -> Result<(), ImportFixtureError> {
		let mut ctx = self.1.write().await;
		if let Some(output_driver) = ctx
			.output_drivers
			.get(&fixture_bundle.fixture_info.output_driver)
		{
			if let Err(controller_error) = output_driver
				.import_fixture(&fixture_bundle.fixture_info.id, fixture_bundle.output_info)
				.await
			{
				return Err(ImportFixtureError::ErrorFromController(controller_error));
			} else {
				// Controller successfully loaded protocol-specific details
				ctx.sharable.library.insert(
					fixture_bundle.fixture_info.id.clone(),
					fixture_bundle.fixture_info,
				);
				self.0
					.emit("patcher.new_fixture".into(), FilterCriteria::None, ())
					.await;
				return Ok(());
			}
		} else {
			return Err(ImportFixtureError::UnknownController);
		}
	}

	/// Gets the form to display for creating a new fixture
	pub async fn get_creation_form(
		&self,
		fixture_type: &Uuid,
	) -> Result<FormDescriptor, GetCreationFormError> {
		let ctx = self.1.read().await;
		if let Some(fixture_info) = ctx.sharable.library.get(&fixture_type) {
			let output_driver = ctx
				.output_drivers
				.get(&fixture_info.output_driver)
				.expect("Found reference to non-existant driver in fixture library");
			return Ok(output_driver.get_creation_form(&fixture_info).await?);
		} else {
			return Err(GetCreationFormError::FixtureTypeMissing);
		}
	}

	/// Gets the form to display for creating a new fixture
	pub async fn get_edit_form(
		&self,
		fixture_id: &Uuid,
	) -> Result<FormDescriptor, GetEditFormError> {
		let ctx = self.1.read().await;
		if let Some(instance_info) = ctx.sharable.fixtures.get(&fixture_id) {
			if let Some(fixture_info) = ctx.sharable.library.get(&instance_info.fixture_id) {
				let output_driver = ctx
					.output_drivers
					.get(&fixture_info.output_driver)
					.expect("Found reference to non-existant driver in fixture library");
				return Ok(output_driver.get_edit_form(&fixture_id).await?);
			} else {
				return Err(GetEditFormError::FixtureDefinitionMissing);
			}
		} else {
			return Err(GetEditFormError::FixtureMissing);
		}
	}

	/// Create a fixture
	pub async fn create_fixture(
		&self,
		fixture_type: Uuid,
		personality: String,
		name: Option<String>,
		comments: Option<String>,
		form_data: SerializedData,
	) -> Result<Uuid, CreateFixtureError> {
		let mut ctx = self.1.write().await;
		if let Some(fixture_type_info) = ctx.sharable.library.get(&fixture_type) {
			if let Some(controller) = ctx.output_drivers.get(&fixture_type_info.output_driver) {
				let instance_uuid = Uuid::new_v4();
				if let Err(controller_error) = controller
					.create_fixture_instance(
						&ctx.sharable,
						&instance_uuid,
						fixture_type_info,
						&personality,
						form_data,
					)
					.await
				{
					return Err(CreateFixtureError::ErrorFromController(controller_error));
				} else {
					// Controller successfully loaded protocol-specific details
					ctx.sharable.fixture_order.push(instance_uuid.clone());
					ctx.sharable.fixtures.insert(
						instance_uuid.clone(),
						FixtureInstance {
							id: instance_uuid.clone(),
							fixture_id: fixture_type,
							personality,
							name,
							comments,
						},
					);
					self.0
						.emit("patcher.patch_updated".into(), FilterCriteria::None, ())
						.await;
					return Ok(instance_uuid);
				}
			} else {
				return Err(CreateFixtureError::ControllerMissing);
			}
		} else {
			return Err(CreateFixtureError::FixtureTypeMissing);
		}
	}

	/// Edit a fixture
	pub async fn edit_fixture(
		&self,
		instance_id: &Uuid,
		personality: String,
		name: Option<String>,
		comments: Option<String>,
		form_data: SerializedData,
	) -> Result<(), EditFixtureError> {
		let mut ctx = self.1.write().await;

		// Need to take ownership in order to mutate ctx. Make sure it gets put back.
		if let Some((instance_id, fixture)) = ctx.sharable.fixtures.remove_entry(instance_id) {
			if let Some(fixture_type_info) = ctx.sharable.library.get(&fixture.fixture_id) {
				if let Some(controller) = ctx.output_drivers.get(&fixture_type_info.output_driver) {
					if let Err(controller_err) = controller
						.edit_fixture_instance(
							&ctx.sharable,
							&instance_id,
							fixture_type_info,
							&personality,
							form_data,
						)
						.await
					{
						// Put the fixture back since we're not replacing it
						ctx.sharable.fixtures.insert(instance_id, fixture);
						return Err(EditFixtureError::ErrorFromController(controller_err));
					} else {
						// Insert the new fixture
						ctx.sharable.fixtures.insert(
							fixture.id,
							FixtureInstance {
								id: instance_id,
								fixture_id: fixture.fixture_id,
								personality,
								name,
								comments,
							},
						);
						self.0
							.emit("patcher.patch_updated".into(), FilterCriteria::None, ())
							.await;
						return Ok(());
					}
				} else {
					// Put the fixture back since we can't edit
					ctx.sharable.fixtures.insert(instance_id.clone(), fixture);
					return Err(EditFixtureError::ControllerMissing);
				}
			} else {
				// Put the fixture back since we can't edit
				ctx.sharable.fixtures.insert(instance_id.clone(), fixture);
				return Err(EditFixtureError::FixtureTypeMissing);
			}
		} else {
			return Err(EditFixtureError::FixtureMissing);
		}
	}

	/// Write values to the output plugins
	pub async fn write_values(&self, data: Arc<FullMixerOutput>) {
		let ctx = self.1.read().await;

		let mut futures = Vec::new();
		for driver in ctx.output_drivers.values().cloned() {
			let state = self.get_sharable_state().await;
			let data = Arc::clone(&data);
			futures.push(async move {
				driver.send_updates(&state, data).fuse().await;
			});
		}
		drop(ctx);
		join_all(futures).await;
	}

	pub async fn get_sharable_state<'a>(&'a self) -> SharableStateWrapper<'a> {
		let ctx = self.1.read().await;
		return SharableStateWrapper::new(ctx);
	}
}

#[async_trait]
impl Savable for PatcherInterface {
	async fn save_data(&self) -> Result<Option<Vec<u8>>, String> {
		let ctx = self.1.read().await;
		return Ok(Some(ctx.sharable.serialize_cbor()?));
	}
}

fn get_max_value(channel_size: &ChannelSize) -> u16 {
	return match channel_size {
		ChannelSize::U8 => 255,
		ChannelSize::U16 => 65535,
	};
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
#[derive(Error)]
/// An error that could occur when importing a fixture definition
pub enum ImportFixtureError {
	#[error("Could not find the controller referenced by the fixture definition")]
	UnknownController,
	#[error("An error occured while importing controller-specific details:\n{0:?}")]
	ErrorFromController(driver_plugin_api::ImportError),
}

#[portable]
#[derive(Error)]
/// An error that could occur when creating a fixture
pub enum CreateFixtureError {
	#[error("The requested fixture type does not exist in the library")]
	FixtureTypeMissing,
	#[error("The controller responsible for this fixture is missing")]
	ControllerMissing,
	#[error("The controller reported an error while creating an instance of the fixture:\n{0:?}")]
	ErrorFromController(driver_plugin_api::CreateInstanceError),
}

#[portable]
#[derive(Error)]
/// An error that could occur while retrieving a fixture creation form
pub enum GetCreationFormError {
	#[error("The requested fixture type does not exist in the library")]
	FixtureTypeMissing,
	#[error("The controller reported an error while getting the creation form:\n{0}")]
	Other(String),
}
impl_anyhow!(GetCreationFormError, GetCreationFormError::Other);

#[portable]
#[derive(Error)]
/// An error that could occur while retrieving a fixture edit form
pub enum GetEditFormError {
	#[error("The requested fixture instance does not exist in the library.")]
	FixtureMissing,
	#[error("The definition for the requested fixture is missing.")]
	FixtureDefinitionMissing,
	#[error("The controller associated with this fixture is missing.")]
	ControllerMissing,
	#[error("The controller reported an error while getting the edit form:\n{0}")]
	ControllerError(String),
}
impl From<anyhow::Error> for GetEditFormError {
	fn from(value: anyhow::Error) -> Self {
		return Self::ControllerError(value.to_string());
	}
}

#[portable]
#[derive(Error)]
/// An error that could occur when creating a fixture
pub enum EditFixtureError {
	#[error("This fixture does not exist")]
	FixtureMissing,
	#[error("The fixture definition is missing for the requested fixture type")]
	FixtureTypeMissing,
	#[error("The controller responsible for this fixture is missing")]
	ControllerMissing,
	#[error("The controller reported an error while creating an instance of the fixture:\n{0:?}")]
	ErrorFromController(driver_plugin_api::EditInstanceError),
}

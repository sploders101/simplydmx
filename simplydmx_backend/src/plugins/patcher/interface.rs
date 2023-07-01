use super::{
	driver_plugin_api::{self, FixtureBundle, FixtureInstance, OutputDriver, SharableStateWrapper},
	fixture_types::{ChannelSize, ChannelType, Segment},
	state::{PatcherContext, VisualizationInfo},
};
use crate::{
	impl_anyhow,
	mixer_utils::state::{BlendingData, FullMixerBlendingData, FullMixerOutput, SnapData},
	plugins::saver::Savable,
	utilities::{forms::FormDescriptor, serialized_data::SerializedData},
};
use async_trait::async_trait;
use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;
use std::sync::Arc;
use thiserror::Error;
use tokio::{sync::RwLock, task::JoinSet};
use uuid::Uuid;

macro_rules! unwrap_continue {
	($opt:expr) => {
		if let Some(tmp) = $opt {
			tmp
		} else {
			continue
		}
	}
}

#[derive(Clone)]
pub struct PatcherInterface(PluginContext, Arc<RwLock<PatcherContext>>);
impl PatcherInterface {
	pub fn new(plugin_context: PluginContext, patcher_ctx: Arc<RwLock<PatcherContext>>) -> Self {
		return PatcherInterface(plugin_context, patcher_ctx);
	}

	/// Gets the initial background layer for the mixer to blend data with.
	pub async fn get_base_layer(&self) -> (FullMixerOutput, FullMixerBlendingData) {
		let mut default_values: FullMixerOutput = FxHashMap::default();
		let mut blending_data: FullMixerBlendingData = FxHashMap::default();

		let ctx = self.1.read().await;

		for (fixture_id, fixture_data) in ctx.sharable.fixtures.iter() {
			if let Some(fixture_info) = ctx.sharable.library.get(&fixture_data.fixture_id) {
				if let Some(fixture_personality) =
					fixture_info.personalities.get(&fixture_data.personality)
				{
					// Create containers for this fixture
					let mut fixture_defaults = FxHashMap::default();
					let mut fixture_blending_data = FxHashMap::default();

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
				self.0
					.emit_if_needed("patcher.new_fixture".into(), FilterCriteria::None, || fixture_bundle.fixture_info.clone())
					.await;
				ctx.sharable.library.insert(
					fixture_bundle.fixture_info.id.clone(),
					fixture_bundle.fixture_info,
				);
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
					let fixture = FixtureInstance {
						id: instance_uuid.clone(),
						fixture_id: fixture_type,
						personality,
						name,
						comments,
						visualization_info: Default::default(),
					};
					self.0
						.emit_if_needed(
							"patcher.patch_updated".into(),
							FilterCriteria::Uuid(instance_uuid.clone()),
							|| Some(fixture.clone()),
						)
						.await;
					// Controller successfully loaded protocol-specific details
					ctx.sharable.fixture_order.push(instance_uuid.clone());
					ctx.sharable.fixtures.insert(
						instance_uuid.clone(),
						fixture,
					);
					return Ok(instance_uuid);
				}
			} else {
				return Err(CreateFixtureError::ControllerMissing);
			}
		} else {
			return Err(CreateFixtureError::FixtureTypeMissing);
		}
	}

	pub async fn delete_fixture(&self, fixture_id: &Uuid) -> Result<(), DeleteFixtureError> {
		let mut ctx = self.1.write().await;

		let (instance_id, fixture) = ctx.sharable.fixtures
			.remove_entry(fixture_id)
			.ok_or(DeleteFixtureError::FixtureMissing)?;
		let fixture_type_info = match ctx.sharable.library.get(&fixture.fixture_id) {
			Some(fixture_type_info) => fixture_type_info,
			None => {
				ctx.sharable.fixtures.insert(instance_id, fixture);
				return Err(DeleteFixtureError::FixtureTypeMissing);
			},
		};
		let controller = match ctx.output_drivers.get(&fixture_type_info.output_driver) {
			Some(controller) => controller,
			None => {
				ctx.sharable.fixtures.insert(instance_id, fixture);
				return Err(DeleteFixtureError::ControllerMissing);
			},
		};

		if let Err(controller_err) = controller.remove_fixture_instance(fixture_id).await {
			// Put the fixture back since the controller refused to remove it
			ctx.sharable.fixtures.insert(instance_id, fixture);
			return Err(DeleteFixtureError::ErrorFromController(controller_err.to_string()));
		} else {
			ctx.sharable.fixture_order.retain(|fixture| fixture != fixture_id);
			self.0
				.emit("patcher.patch_updated".into(), FilterCriteria::Uuid(fixture_id.clone()), Option::<FixtureInstance>::None)
				.await;
			return Ok(());
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
		// Need to take ownership in order to mutate fixtures due to lifetime constraints. Make sure it gets put back.
		let mut ctx = self.1.write().await;

		let (instance_id, fixture) = ctx.sharable.fixtures
			.remove_entry(instance_id)
			.ok_or(EditFixtureError::FixtureMissing)?;
		let fixture_type_info = match ctx.sharable.library.get(&fixture.fixture_id) {
			Some(fixture_type_info) => fixture_type_info,
			None => {
				ctx.sharable.fixtures.insert(instance_id, fixture);
				return Err(EditFixtureError::FixtureTypeMissing);
			},
		};
		let controller = match ctx.output_drivers.get(&fixture_type_info.output_driver) {
			Some(controller) => controller,
			None => {
				ctx.sharable.fixtures.insert(instance_id, fixture);
				return Err(EditFixtureError::ControllerMissing);
			},
		};

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
			let new_instance = FixtureInstance {
				id: instance_id,
				fixture_id: fixture.fixture_id,
				personality,
				name,
				comments,
				visualization_info: fixture.visualization_info,
			};
			self.0
				.emit_if_needed(
					"patcher.patch_updated".into(),
					FilterCriteria::Uuid(fixture.id.clone()),
					|| Some(new_instance.clone()),
				)
				.await;
			ctx.sharable.fixtures.insert(
				fixture.id,
				new_instance,
			);
			return Ok(());
		}
	}

	/// Edits the position of a fixture within the visualizer
	pub async fn edit_fixture_placement(&self, instance_id: &Uuid, x: u16, y: u16) {
		let mut ctx = self.1.write().await;

		if let Some(fixture) = ctx.sharable.fixtures.get_mut(instance_id) {
			fixture.visualization_info.x = x;
			fixture.visualization_info.y = y;
		}

		self.0.emit_if_needed(
			"patcher.visualization_updated".into(),
			FilterCriteria::Uuid(instance_id.clone()), 
			|| VisualizationInfo {
				x,
				y,
			},
		).await;
	}

	/// Gets the position of a fixture within the visualizer
	pub async fn get_fixture_placement(&self, instance_id: &Uuid) -> Option<(u16, u16)> {
		let ctx = self.1.read().await;

		if let Some(fixture) = ctx.sharable.fixtures.get(instance_id) {
			return Some((fixture.visualization_info.x, fixture.visualization_info.y));
		} else {
			return None;
		}
	}

	/// Applies any virtual intensity channels defined in each fixture's
	/// type definition
	fn apply_virtual_intensities(
		data: &FxHashMap<Uuid, FxHashMap<String, u16>>,
		ctx: &PatcherContext,
	) -> Arc<FxHashMap<Uuid, FxHashMap<String, u16>>> {
		let mut new_data = data.clone();

		// Loop over every fixture in immutable data
		for (fixture_id, fixture_values) in data {
			let fixture_instance = unwrap_continue!(ctx.sharable.fixtures.get(fixture_id));
			let fixture_type = unwrap_continue!(ctx.sharable.library.get(&fixture_instance.fixture_id));
			let fixture_personality = unwrap_continue!(fixture_type
				.personalities
				.get(&fixture_instance.personality));
			let new_fixture_values = unwrap_continue!(new_data.get_mut(fixture_id));

			// Loop over every channel in the current fixture (within immutable data)
			for channel_id in fixture_personality.available_channels.iter() {
				let channel = unwrap_continue!(fixture_type.channels.get(channel_id));
				let channel_value = *unwrap_continue!(fixture_values.get(channel_id));
				let inhibited_channels = unwrap_continue!(&channel.intensity_emulation);

				// Loop over every channel inhibited by this one, which we have by now determined
				// to be a virtual intensity channel (otherwise `continue` would have been called by now)
				for inhibited_channel_id in inhibited_channels {
					let inhibited_channel =
						unwrap_continue!(fixture_type.channels.get(inhibited_channel_id));
					let inhibited_channel_value =
						*unwrap_continue!(fixture_values.get(inhibited_channel_id));
					new_fixture_values.insert(
						inhibited_channel_id.clone(),

						// Different variations of the blending algorithm are needed based on channel size
						// combination
						match channel.size {
							ChannelSize::U8 => match inhibited_channel.size {
								ChannelSize::U8 => inhibited_channel_value * channel_value / 255u16,
								ChannelSize::U16 => {
									(inhibited_channel_value as u32 * channel_value as u32 / 255u32)
										as u16
								}
							},
							ChannelSize::U16 => {
								(inhibited_channel_value as u32 * channel_value as u32 / 65535u32)
									as u16
							}
						},
					);
				}
			}
		}
		return Arc::new(new_data);
	}

	/// Write values to the output plugins
	pub async fn write_values(&self, data: Arc<FullMixerOutput>) {
		let ctx = self.1.read().await;

		let data = Self::apply_virtual_intensities(&data, &ctx);

		let mut futures = JoinSet::new();
		for driver in ctx.output_drivers.values().cloned() {
			let owned_self_ref = self.clone();
			let data = Arc::clone(&data);
			futures.spawn(async move {
				let state = owned_self_ref.get_sharable_state().await;
				driver.send_updates(&state, data).await;
			});
		}
		drop(ctx);
		while let Some(result) = futures.join_next().await {
			// Bubble panics into this thread. Panics mean UB has been reached,
			// and the entire system is compromised.
			result.unwrap();
		}
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
	#[error("[Internal state error]: The fixture definition is missing for the requested fixture type")]
	FixtureTypeMissing,
	#[error("[Internal state error]: The controller responsible for this fixture is missing")]
	ControllerMissing,
	#[error("The controller reported an error while creating an instance of the fixture:\n{0:?}")]
	ErrorFromController(driver_plugin_api::EditInstanceError),
}

#[portable]
#[derive(Error)]
/// An error that could occur when removing a fixture
pub enum DeleteFixtureError {
	#[error("This fixture does not exist")]
	FixtureMissing,
	#[error("[Internal state error]: The fixture definition is missing for the requested fixture type.")]
	FixtureTypeMissing,
	#[error("[Internal state error]: The controller responsible for this fixture is missing")]
	ControllerMissing,
	#[error("The controller reported an error while creating an instance of the fixture:\n{0}")]
	ErrorFromController(String),
}

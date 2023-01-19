use std::collections::HashMap;
use async_std::sync::{
	Arc,
	RwLock,
};
use futures::{
	future::join_all,
	FutureExt,
};
use uuid::Uuid;
use async_trait::async_trait;
use anyhow::{anyhow, Context};
use simplydmx_plugin_framework::*;
use crate::{
	plugins::{
		patcher::driver_plugin_api::*,
		saver::Savable,
	},
	mixer_utils::state::{
		FullMixerOutput,
		FixtureMixerOutput,
	},
	utilities::{
		serialized_data::SerializedData,
		forms::{
			InteractiveDescription,
			NumberValidation,
		},
	},
};
use super::{
	state::{
		DMXState,
		UniverseInstance,
		DMXFixtureInstance,
	},
	driver_types::{
		DMXDriver,
		DMXFrame,
		RegisterUniverseError,
	},
	fixture_types::DMXFixtureData,
};


#[derive(Clone)]
pub struct DMXInterface(PluginContext, Arc::<RwLock::<DMXState>>);
impl DMXInterface {
	pub fn new(plugin_context: PluginContext) -> Self {
		return DMXInterface(plugin_context, Arc::new(RwLock::new(DMXState::new())));
	}
	pub fn from_file(plugin_context: PluginContext, file: DMXShowSave) -> Self {
		return DMXInterface(plugin_context, Arc::new(RwLock::new(DMXState::from_file(file))));
	}

	/// Registers an DMX driver for sending universe frames
	pub async fn register_dmx_driver<T: DMXDriver>(&self, plugin: T) {
		let mut ctx = self.1.write().await;
		ctx.drivers.insert(plugin.get_id(), Arc::new(Box::new(plugin)));
	}

	/// Creates a new universe
	pub async fn create_universe(&self, name: String) -> Uuid {
		let mut ctx = self.1.write().await;
		let new_id = Uuid::new_v4();
		ctx.universes.insert(new_id.clone(), UniverseInstance {
			id: new_id.clone(),
			name,
			controller: None,
		});
		return new_id;
	}

	/// Delete a universe from the registry
	pub async fn delete_universe(&self, universe_id: Uuid) {
		let mut ctx = self.1.write().await;
		if ctx.universes.contains_key(&universe_id) {
			// Unlink fixtures from universe
			for fixture_info in ctx.fixtures.values_mut() {
				if fixture_info.universe == Some(universe_id) {
					fixture_info.universe = None;
					fixture_info.offset = None;
				}
			}

			// Unlink universe from controller
			if let Some(universe_data) = ctx.universes.get(&universe_id) {
				if let Some(ref driver_id) = universe_data.controller {
					if let Some(ref driver) = ctx.drivers.get(driver_id) {
						driver.delete_universe(&universe_id).await;
					}
				}
			}

			// Delete universe
			ctx.universes.remove(&universe_id);

			// Emit event for any plugins that care
			self.0.emit("dmx.universe_removed".into(), FilterCriteria::Uuid(universe_id), ()).await;
		}
	}

	/// Links an existing universe to a driver
	pub async fn link_universe(
		&self,
		universe_id: &Uuid,
		driver: String,
		form_data: SerializedData,
	) -> Result<(), LinkUniverseError> {
		let mut ctx = self.1.write().await;
		if ctx.universes.contains_key(universe_id) {
			if let Some(controller) = ctx.drivers.get(&driver) {
				if let Err(err) = controller.register_universe(universe_id, form_data).await {
					return Err(LinkUniverseError::ErrorFromController(err));
				}
			} else {
				return Err(LinkUniverseError::ControllerNotFound);
			}
		} else {
			return Err(LinkUniverseError::UniverseNotFound);
		}

		ctx.universes.get_mut(universe_id).unwrap().controller = Some(driver);
		return Ok(());
	}

	/// Unlinks an existing universe from its driver
	pub async fn unlink_universe(&self, universe_id: &Uuid) {
		let mut ctx = self.1.write().await;
		let mut existing_controller = None;
		if let Some(universe) = ctx.universes.get_mut(universe_id) {
			existing_controller = universe.controller.clone();
			universe.controller = None;
		}
		if let Some(ref controller_id) = existing_controller {
			if let Some(controller) = ctx.drivers.get(controller_id) {
				controller.delete_universe(universe_id).await;
			}
		}
	}

	pub async fn list_universes(&self) -> Vec<(Uuid, String)> {
		let ctx = self.1.write().await;
		return ctx.universes.values().map(|universe| (universe.id.clone(), universe.name.clone())).collect();
	}
}

#[portable]
/// The DMX portion of the show file
pub struct DMXShowSave {
	pub library: HashMap<Uuid, DMXFixtureData>,
	pub fixtures: HashMap<Uuid, DMXFixtureInstance>,
	pub universes: HashMap<Uuid, UniverseInstance>,
}

#[async_trait]
impl Savable for DMXInterface {
	async fn save_data(&self) -> Result<Option<Vec<u8>>, String> {
		let ctx = self.1.read().await;
		return Ok(Some(DMXShowSave {
			library: ctx.library.clone(), // TODO: Minify this first
			fixtures: ctx.fixtures.clone(),
			universes: ctx.universes.clone(),
		}.serialize_cbor()?));
	}
}


// Implement the patcher's OutputPlugin trait for the DMX plugin's interface object
#[async_trait]
impl OutputDriver for DMXInterface {


	// Metadata
	fn get_id(&self) -> String {
		return "DMX".into();
	}
	fn get_name(&self) -> String {
		return "DMX".into();
	}
	fn get_description(&self) -> String {
		return "Controller for the DMX protocol. This plugin handles the creation of DMX frames and passes them off to a driver plugin.".into();
	}


	// Fixture library imports/exports

	/// Takes a `Uuid` and a `DMXFixtureData` instance serialized as `SerializedData`
	async fn import_fixture(&self, id: &Uuid, data: SerializedData) -> Result<(), ImportError> {
		let mut ctx = self.1.write().await;
		ctx.library.insert(id.clone(), data.deserialize()?);
		return Ok(());
	}

	async fn export_fixture_json(&self, id: &Uuid) -> anyhow::Result<serde_json::Value> {
		let ctx = self.1.read().await;
		let fixture = ctx.library.get(&id).with_context(|| format!("Could not find fixture {id}"))?;
		let serialized = serde_json::to_value(fixture)
			.with_context(|| format!("An error occurred while serializing fixture {id}"))?;
		return Ok(serialized);
	}

	async fn export_fixture_cbor(&self, id: &Uuid) -> anyhow::Result<Vec<u8>> {
		let ctx = self.1.read().await;
		let fixture = ctx.library.get(&id).with_context(|| format!("Could not find fixture {id}"))?;
		let mut serialized = Vec::<u8>::new();
		ciborium::ser::into_writer(fixture, &mut serialized)
			.with_context(|| format!("An error occurred while serializing fixture {id}."))?;
		return Ok(serialized);
	}


	// Fixture creation/removal

	async fn get_creation_form(&self, _fixture_info: &FixtureInfo) -> anyhow::Result<FormDescriptor> {
		return Ok(FormDescriptor::new()
			.dropdown_dynamic("Universe", "universe", "universes_optional")
			.dynamic(
				InteractiveDescription::not(InteractiveDescription::Equal {
					field_name: "universe".into(),
					value: serde_json::Value::Null,
				}),
				|form| form.number_prefilled("DMX Offset", "offset", NumberValidation::And(vec![
					NumberValidation::Between(1.0, 512.0),
					NumberValidation::DivisibleBy(1.0),
				]), 1.0),
			)
			.build());
	}

	async fn create_fixture_instance(
		&self,
		patcher: &SharablePatcherState,
		id: &Uuid,
		fixture_type_info: &FixtureInfo,
		personality_id: &str,
		form: SerializedData,
	) -> Result<(), CreateInstanceError> {
		let mut ctx = self.1.write().await;
		let normalized_fixture = ctx.normalize_fixture(patcher, fixture_type_info, personality_id, form.deserialize()?).await?;
		ctx.fixtures.insert(id.clone(), normalized_fixture);
		return Ok(());
	}

	async fn remove_fixture_instance(&self, id: &Uuid) -> anyhow::Result<()> {
		let mut ctx = self.1.write().await;
		ctx.fixtures.remove(id);
		return Ok(());
	}


	// Fixture editing
	async fn get_edit_form(&self, instance_id: &Uuid) -> anyhow::Result<FormDescriptor> {
		let ctx = self.1.read().await;
		let fixture_info = ctx.fixtures.get(instance_id)
			.map_or_else(|| Err(anyhow!("Couldn't get fixture instance")), |instance| Ok(instance))?;
		return Ok(FormDescriptor::new()
			.dropdown_dynamic_prefilled("Universe", "universe", "universes_optional", fixture_info.universe.serialize_json()?)
			.dynamic(
				InteractiveDescription::not(InteractiveDescription::Equal {
					field_name: "universe".into(),
					value: serde_json::Value::Null,
				}),
				|form| form.number_prefilled("DMX Offset", "offset", NumberValidation::And(vec![
					NumberValidation::Between(1.0, 512.0),
					NumberValidation::DivisibleBy(1.0),
				]), fixture_info.offset.map_or(1.0, |offset| offset as f64)),
			)
			.build());
	}

	async fn edit_fixture_instance(&self, id: &Uuid, form: SerializedData) -> Result<(), EditError> {
		let mut ctx = self.1.write().await;
		ctx.fixtures.insert(id.clone(), form.deserialize()?);
		return Ok(());
	}


	// Updates

	async fn send_updates<'a>(&self, patcher_data: &'a SharableStateWrapper<'a>, data: Arc<FullMixerOutput>) {
		let ctx = self.1.read().await;

		// Create default universes
		let mut universes = HashMap::<Uuid, DMXFrame>::from_iter(
			ctx.universes.keys()
				.map(|universe_id| (universe_id.clone(), [0u8; 512]))
		);

		// Add fixtures to universes
		for (fixture_instance_id, fixture_instance_data) in ctx.fixtures.iter() {
			if let (
				Some(fixture_mixer_data),
				Some(patcher_fixture_instance),
				Some(ref universe_id),
				Some(offset)
			) = (
				data.get(fixture_instance_id),
				patcher_data.fixtures.get(fixture_instance_id),
				fixture_instance_data.universe,
				fixture_instance_data.offset,
			) {
				if let (
					Some(mut universe_frame),
					Some(patcher_fixture_type),
					Some(fixture_type),
				) = (
					universes.get_mut(universe_id),
					patcher_data.library.get(&patcher_fixture_instance.fixture_id),
					ctx.library.get(&patcher_fixture_instance.fixture_id),
				) {
					insert_fixture_data(
						patcher_fixture_instance,
						patcher_fixture_type,
						// fixture_instance_data,
						fixture_type,
						&fixture_mixer_data,
						offset,
						&mut universe_frame,
					);
				}
			}
		}
		drop(patcher_data);

		// Sort universes into designated controller-centric HashMaps
		let mut sorted_universes = HashMap::<String, HashMap::<Uuid, DMXFrame>>::new();
		for (universe_id, universe_data) in ctx.universes.iter() {
			if let Some(universe_frame) = universes.remove(universe_id) {
				// Send an event for the UI with the dmx output data, useful for inspectors
				self.0.emit("dmx.output".into(), FilterCriteria::Uuid(universe_id.clone()), universe_frame.to_vec()).await;
				if let Some(ref controller_id) = universe_data.controller {
					// Create a universe collection for the controller if it doesn't already have one
					if !sorted_universes.contains_key(controller_id) {
						sorted_universes.insert(String::clone(controller_id), HashMap::new());
					}

					// Insert the current universe into its controller's collection
					sorted_universes.get_mut(controller_id).unwrap().insert(universe_id.clone(), universe_frame);
				}
			}
		}

		// Spawn a task for each controller, sending it relevant DMX data
		let mut futures = Vec::new();
		for (controller_id, universes) in sorted_universes {
			if let Some(controller) = ctx.drivers.get(&controller_id) {
				let controller = Arc::clone(controller);
				futures.push(async move {
					controller.send_dmx(universes).await;
				}.fuse());
			}
		}
		join_all(futures).await;
	}


}


/// Renders a fixture's data into the universe. If fixtures are configured improperly, this function will incur a race condition.
///
/// Overlaps should be caught before this point
fn insert_fixture_data(
	patcher_fixture_instance: &FixtureInstance,
	patcher_fixture_type: &FixtureInfo,
	// fixture_data: &DMXFixtureInstance,
	fixture_type: &DMXFixtureData,
	data: &FixtureMixerOutput,
	mut offset: u16,
	universe_frame: &mut DMXFrame,
) {
	if let Some(dmx_personality) = fixture_type.personalities.get(&patcher_fixture_instance.personality) {
		for channel_name in dmx_personality.dmx_channel_order.iter() {
			if let (
				Some(channel_info),
				Some(channel_value),
			) = (
				patcher_fixture_type.channels.get(channel_name),
				data.get(channel_name),
			) {
				match channel_info.size {
					ChannelSize::U8 => {
						universe_frame[(offset - 1) as usize] = *channel_value as u8;
						offset += 1;
					},
					ChannelSize::U16 => {
						let current_value_bytes = channel_value.to_be_bytes();
						universe_frame[(offset - 1) as usize] = current_value_bytes[0];
						universe_frame[offset as usize] = current_value_bytes[1];
						offset += 2;
					},
				}
			}
		}
	}
}

#[portable]
#[serde(tag = "type", content = "data")]
/// An error that could occur while linking a DMX universe to a universe controller
pub enum LinkUniverseError {
	ErrorFromController(RegisterUniverseError),
	UniverseNotFound,
	ControllerNotFound,
}

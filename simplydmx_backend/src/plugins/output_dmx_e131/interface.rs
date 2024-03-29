use super::{
	dmxsource_controller::initialize_controller,
	state::{E131State, E131Universe},
};
use crate::utilities::forms::NumberValidation;
use crate::{
	plugins::{output_dmx::driver_types::*, patcher::driver_plugin_api::*, saver::Savable},
	utilities::serialized_data::SerializedData,
};
use async_trait::async_trait;
use simplydmx_plugin_framework::*;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct E131DMXDriver(PluginContext, Arc<RwLock<E131State>>);
impl E131DMXDriver {
	pub async fn new(plugin_context: PluginContext) -> Self {
		return E131DMXDriver(
			plugin_context.clone(),
			Arc::new(RwLock::new(E131State::new(
				initialize_controller(plugin_context).await,
			))),
		);
	}
	pub async fn from_file(plugin_context: PluginContext, file: E131DMXShowSave) -> Self {
		return E131DMXDriver(
			plugin_context.clone(),
			Arc::new(RwLock::new(E131State::from_file(
				initialize_controller(plugin_context).await,
				file,
			))),
		);
	}

	async fn create_universe(self, int_id: Uuid, data: E131Universe) -> Result<(), &'static str> {
		let mut ctx = self.1.write().await;

		if let Some(_) = ctx
			.universes
			.values()
			.find(|universe| universe.external_universe == data.external_universe)
		{
			return Err("This external universe ID is taken");
		}

		if ctx.universes.len() == 0 {
			let mut controller_lock = ctx.controller.lock().await;
			if let None = *controller_lock {
				*controller_lock = Some(HashMap::new());
			}
		}
		ctx.universes.insert(int_id, data);

		return Ok(());
	}

	async fn destroy_universe(self, int_id: Uuid) -> () {
		let mut ctx = self.1.write().await;

		if let Some(ext_data) = ctx.universes.remove(&int_id) {
			let mut controller = ctx.controller.lock().await;
			if ctx.universes.len() == 0 {
				*controller = None;
			} else {
				if let Some(ref mut controller) = *controller {
					controller.remove(&ext_data.external_universe);
				}
			}
		}
	}

	async fn send_frames(self, frames: HashMap<Uuid, DMXFrame>) -> () {
		let ctx = self.1.read().await;
		let mut dereferenced_data = HashMap::<u16, [u8; 512]>::new();

		// Build hashmap of sACN IDs -> DMXFrames
		for (internal_id, frame) in frames {
			if let Some(external_data) = ctx.universes.get(&internal_id) {
				dereferenced_data.insert(external_data.external_universe, frame);
			}
		}

		// Push new hashmap to sacn controller
		let mut controller_lock = ctx.controller.lock().await;
		if dereferenced_data.len() > 0 {
			*controller_lock = Some(dereferenced_data);
		} else {
			*controller_lock = None;
		}
	}
}

#[portable]
/// The E131 portion of the show file
pub struct E131DMXShowSave {
	pub universes: HashMap<Uuid, E131Universe>,
}

#[async_trait]
impl Savable for E131DMXDriver {
	async fn save_data(&self) -> Result<Option<Vec<u8>>, String> {
		let ctx = self.1.read().await;
		return Ok(Some(
			E131DMXShowSave {
				universes: ctx.universes.clone(),
			}
			.serialize_cbor()?,
		));
	}
}

#[async_trait]
impl DMXDriver for E131DMXDriver {
	/// The unique ID of the DMX driver
	fn get_id<'a>(&'a self) -> &'a str {
		return "e131";
	}

	/// The human-readable name of the DMX driver
	fn get_name<'a>(&'a self) -> &'a str {
		return "E.131/sACN";
	}

	/// A human-readable description of the driver, such as what devices and protocols it uses
	fn get_description<'a>(&'a self) -> &'a str {
		return "Controls an E.131/sACN universe attached to the network";
	}

	/// Gets a form used by the UI for linking a universe to this driver
	async fn get_register_universe_form(&self, universe_id: Option<&Uuid>) -> anyhow::Result<FormDescriptor> {
		let ctx = if universe_id.is_some() { Some(self.1.read().await) } else { None };
		let universe = if let Some(ref ctx) = ctx { ctx.universes.get(universe_id.unwrap()) } else { None };
		return Ok(FormDescriptor::new().number_prefilled(
			"E.131 universe ID",
			"external_universe",
			NumberValidation::And(vec![
				NumberValidation::Between(1.0, 63999.0),
				NumberValidation::DivisibleBy(1.0),
			]),
			universe
				.and_then(|universe| Some(universe.external_universe as f64))
				.unwrap_or(1.0),
		));
	}

	/// Registers a universe using data from a filled-in form
	async fn register_universe(
		&self,
		id: &Uuid,
		form: SerializedData,
	) -> Result<(), RegisterUniverseError> {
		let ctx = self.clone();
		if let Err(error) = ctx.create_universe(id.clone(), form.deserialize()?).await {
			return Err(RegisterUniverseError::Other(error.into()));
		}
		return Ok(());
	}

	/// Deletes a universe from the driver
	async fn delete_universe(&self, id: &Uuid) {
		let ctx = self.clone();
		ctx.destroy_universe(id.clone()).await;
	}

	/// Sends new, updated frames to the driver for output
	async fn send_dmx(&self, universes: HashMap<Uuid, [u8; 512]>) {
		let ctx = self.clone();
		ctx.send_frames(universes).await;
	}
}

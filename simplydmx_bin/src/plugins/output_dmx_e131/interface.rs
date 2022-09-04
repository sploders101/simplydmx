use std::collections::HashMap;

use async_std::sync::{
	Arc,
	RwLock,
};

use uuid::Uuid;
use async_trait::async_trait;

use simplydmx_plugin_framework::*;
use crate::{
	plugins::{
		patcher::driver_plugin_api::*,
		output_dmx::driver_types::*,
	},
	utilities::serialized_data::SerializedData,
};

use super::{
	dmxsource_controller::E131Command,
	state::{
		E131State,
		E131Universe,
	},
};


#[derive(Clone)]
pub struct E131DMXDriver(PluginContext, Arc::<RwLock::<E131State>>);
impl E131DMXDriver {
	pub fn new(plugin_context: PluginContext) -> Self {
		return E131DMXDriver(plugin_context, Arc::new(RwLock::new(E131State::new())));
	}

	async fn create_universe(self, int_id: Uuid, data: E131Universe) -> Result<(), &'static str> {
		let mut ctx = self.1.write().await;

		if let Some(_) = ctx.universes.values().find(|universe| universe.external_universe == data.external_universe) {
			return Err("This external universe ID is taken");
		}

		if ctx.universes.len() == 0 {
			if let Err(_) = ctx.controller.send(E131Command::CreateOutput).await {
				log_error!(self.0, "The E.131 controller exited early!");
			}
		}
		ctx.universes.insert(int_id, data);

		return Ok(());
	}

	async fn destroy_universe(self, int_id: Uuid) -> () {
		let mut ctx = self.1.write().await;

		if let Some(ext_data) = ctx.universes.remove(&int_id) {
			if let Err(_) = ctx.controller.send(E131Command::TerminateUniverse(ext_data.external_universe)).await {
				log_error!(self.0, "The E.131 controller exited early!");
			}
			if ctx.universes.len() == 0 {
				if let Err(_) = ctx.controller.send(E131Command::DestroyOutput).await {
					log_error!(self.0, "The E.131 controller exited early!");
				}
			}
		}
	}

	async fn send_frames(self, frames: HashMap<Uuid, DMXFrame>) -> () {
		let ctx = self.1.read().await;
		let mut dereferenced_data = HashMap::<u16, [u8; 512]>::new();
		for (internal_id, frame) in frames {
			if let Some(external_data) = ctx.universes.get(&internal_id) {
				dereferenced_data.insert(external_data.external_universe, frame);
			}
		}
		if let Err(_) = ctx.controller.send(E131Command::SendOutput(dereferenced_data)).await {
			log_error!(self.0, "The E.131 controller exited early!");
		}
	}

}

#[async_trait]
impl DMXDriver for E131DMXDriver {

	/// The unique ID of the DMX driver
	fn get_id(&self) -> String {
		return "e131".into();
	}

	/// The human-readable name of the DMX driver
	fn get_name(&self) -> String {
		return "E.131/sACN".into();
	}

	/// A human-readable description of the driver, such as what devices and protocols it uses
	fn get_description(&self) -> String {
		return "Controls an E.131/sACN universe attached to the network".into();
	}

	/// Gets a form used by the UI for linking a universe to this driver
	async fn get_register_universe_form(&self) -> FormDescriptor {
		return FormDescriptor::new();
	}

	/// Registers a universe using data from a filled-in form
	async fn register_universe(&self, id: &Uuid, form: SerializedData) -> Result<(), RegisterUniverseError> {
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

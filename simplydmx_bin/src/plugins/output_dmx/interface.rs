use async_std::sync::{
	Arc,
	RwLock,
};

use uuid::Uuid;
use async_trait::async_trait;

use simplydmx_plugin_framework::*;
use crate::{plugins::patcher::driver_plugin_api::*, utilities::serialized_data::SerializedData};
use super::{
	state::DMXState,
	driver_types::DMXDriver,
};


#[derive(Clone)]
pub struct DMXInterface(PluginContext, Arc::<RwLock::<DMXState>>);
impl DMXInterface {
	pub fn new(plugin_context: PluginContext) -> Self {
		return DMXInterface(plugin_context, Arc::new(RwLock::new(DMXState::new())));
	}

	/// Registers an output plugin for use by the patcher.
	pub async fn register_output<T: DMXDriver>(&self, plugin: T) {
		let mut ctx = self.1.write().await;
		ctx.drivers.insert(plugin.get_id(), Box::new(plugin));
	}
}


// Implement the patcher's OutputPlugin trait for the DMX plugin's interface object
#[async_trait]
impl OutputPlugin for DMXInterface {


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

	async fn import_fixture(&self, id: &Uuid, data: SerializedData) -> Result<(), ImportError> {
		let mut ctx = self.1.write().await;
		ctx.library.insert(id.clone(), data.deserialize()?);
		return Ok(());
	}

	async fn export_fixture_json(&self, id: &Uuid) -> Option<serde_json::Value> {
		let ctx = self.1.read().await;
		if let Some(fixture) = ctx.library.get(&id) {
			if let Ok(serialized) = serde_json::to_value(fixture) {
				return Some(serialized);
			} else {
				return None;
			}
		} else {
			return None;
		}
	}

	async fn export_fixture_bincode(&self, id: &Uuid) -> Option<Vec<u8>> {
		let ctx = self.1.read().await;
		if let Some(fixture) = ctx.library.get(&id) {
			if let Ok(serialized) = bincode::serialize(fixture) {
				return Some(serialized);
			} else {
				return None;
			}
		} else {
			return None;
		}
	}


	// Fixture creation/removal

	async fn get_creation_form(&self) -> FormDescriptor {
		// TODO
		return FormDescriptor::new();
	}

	async fn create_fixture_instance(&self, id: &Uuid, form: SerializedData) -> Result<(), CreateInstanceError> {
		let mut ctx = self.1.write().await;
		ctx.fixtures.insert(id.clone(), form.deserialize()?);
		return Ok(());
	}

	async fn remove_fixture_instance(&self, id: &Uuid) {
		let mut ctx = self.1.write().await;
		ctx.fixtures.remove(id);
	}


	// Fixture editing

	async fn get_edit_form(&self) -> FormDescriptor {
		// TODO
		return FormDescriptor::new();
	}

	async fn edit_fixture_instance(&self, id: &Uuid, form: SerializedData) -> Result<(), EditError> {
		let mut ctx = self.1.write().await;
		ctx.fixtures.insert(id.clone(), form.deserialize()?);
		return Ok(());
	}


}

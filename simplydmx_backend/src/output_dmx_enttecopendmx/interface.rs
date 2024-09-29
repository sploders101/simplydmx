use crate::{
	plugins::{output_dmx::driver_types::*, patcher::driver_plugin_api::*, saver::Savable},
	utilities::serialized_data::SerializedData,
};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use simplydmx_plugin_framework::*;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

use super::controller::OpenDMXController;

pub struct OpenDMXState {
	universe_id: Option<Uuid>,
	controller: Option<OpenDMXController>,
}
impl Default for OpenDMXState {
	fn default() -> Self {
		return OpenDMXState {
			universe_id: None,
			controller: None,
		};
	}
}

#[derive(Clone)]
pub struct OpenDMXDriver(PluginContext, Arc<Mutex<OpenDMXState>>);
impl OpenDMXDriver {
	pub async fn new(plugin_context: PluginContext) -> Self {
		return OpenDMXDriver(
			plugin_context.clone(),
			Arc::new(Mutex::new(OpenDMXState {
				universe_id: None,
				controller: None,
			})),
		);
	}

	pub async fn from_file(plugin_context: PluginContext, file: OpenDMXShowSave) -> Self {
		return OpenDMXDriver(
			plugin_context.clone(),
			Arc::new(Mutex::new(OpenDMXState {
				universe_id: file.universe_id,
				controller: None,
			})),
		);
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenDMXShowSave {
	universe_id: Option<Uuid>,
}

#[async_trait]
impl Savable for OpenDMXDriver {
	async fn save_data(&self) -> Result<Option<Vec<u8>>, String> {
		let ctx = self.1.lock().await;
		return Ok(Some(
			OpenDMXShowSave {
				universe_id: ctx.universe_id,
			}
			.serialize_cbor()?,
		));
	}
}

#[async_trait]
impl DMXDriver for OpenDMXDriver {
	/// The unique ID of the DMX driver
	fn get_id<'a>(&'a self) -> &'a str {
		return "opendmx";
	}

	/// The human-readable name of the DMX driver
	fn get_name<'a>(&'a self) -> &'a str {
		return "Enttec OpenDMX";
	}

	/// A human-readable description of the driver, such as what devices and protocols it uses
	fn get_description<'a>(&'a self) -> &'a str {
		return "Controls a DMX universe attached to an Enttec OpenDMX USB adapter";
	}

	/// Gets a form used by the UI for linking a universe to this driver
	async fn get_register_universe_form(&self, _universe_id: Option<&Uuid>) -> anyhow::Result<FormDescriptor> {
		return Ok(FormDescriptor::new());
	}

	/// Registers a universe using data from a filled-in form
	async fn register_universe(&self, id: &Uuid, _form: SerializedData) -> Result<(), RegisterUniverseError> {
		let mut ctx = self.1.lock().await;
		match ctx.universe_id {
			Some(ref existing_id) => {
				if id != existing_id {
					return Err(RegisterUniverseError::Other("OpenDMX driver is already in use".into()));
				}
			},
			None => {
				ctx.universe_id = Some(id.clone());
			},
		}
		return Ok(());
	}

	/// Deletes a universe from the driver
	async fn delete_universe(&self, id: &Uuid) {
		let mut ctx = self.1.lock().await;
		if let Some(old_id) = ctx.universe_id {
			if &old_id == id {
				ctx.universe_id = None;
				let old_controller = std::mem::take(&mut ctx.controller);
				if let Some(old_controller) = old_controller {
					old_controller.shutdown().await;
				}
			}
		}
	}

	/// Sends new, updated frames to the driver for output
	async fn send_dmx(&self, mut universes: HashMap<Uuid, [u8; 512]>) {
		let mut ctx = self.1.lock().await;
		match (ctx.universe_id, ctx.controller.as_ref()) {
			(Some(universe_id), Some(controller)) => {
				if let Some(frame) = universes.remove(&universe_id) {
					controller.send_frame(frame).await;
				}
			}
			(Some(universe_id), None) => {
				if let Some(frame) = universes.remove(&universe_id) {
					ctx.controller = Some(OpenDMXController::new(frame));
				}
			}
			(None, Some(_)) | (None, None) => {}
		}
	}
}

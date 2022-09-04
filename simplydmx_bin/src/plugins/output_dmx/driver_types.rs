use std::collections::HashMap;

use simplydmx_plugin_framework::*;
use uuid::Uuid;
use async_trait::async_trait;

use crate::{
	utilities::{
		forms::FormDescriptor,
		serialized_data::SerializedData,
	},
	impl_deserialize_err,
};


/// Trait indicating parameters for communicating with a DMX driver
#[async_trait]
pub trait DMXDriver: Send + Sync + 'static {

	/// The unique ID of the DMX driver
	fn get_id(&self) -> String;

	/// The human-readable name of the DMX driver
	fn get_name(&self) -> String;

	/// A human-readable description of the driver, such as what devices and protocols it uses
	fn get_description(&self) -> String;

	/// Gets a form used by the UI for linking a universe to this driver
	async fn get_register_universe_form(&self) -> FormDescriptor;

	/// Registers a universe using data from a filled-in form
	async fn register_universe(&self, id: &Uuid, form: SerializedData) -> Result<(), RegisterUniverseError>;

	/// Deletes a universe from the driver
	async fn delete_universe(&self, id: &Uuid);

	/// Sends new, updated frames to the driver for output
	async fn send_dmx(&self, universes: HashMap<Uuid, [u8; 512]>);

}

/// Minified representation of a DMX driver for display
#[portable]
pub struct DisplayableDMXDriver {
	pub id: String,
	pub name: String,
	pub description: String,
}

/// A single collection of values to send to a DMX universe
pub type DMXFrame = [u8; 512];

/// An error that occurs while registering a universe
#[portable]
pub enum RegisterUniverseError {
	InvalidData,
	Other(String),
	Unknown,
}
impl_deserialize_err!(RegisterUniverseError, Self::InvalidData);

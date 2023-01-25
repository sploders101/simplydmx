use crate::{
	impl_deserialize_err,
	utilities::{forms::FormDescriptor, serialized_data::SerializedData},
};
use async_trait::async_trait;
use simplydmx_plugin_framework::*;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Trait indicating parameters for communicating with a DMX driver
#[async_trait]
pub trait DMXDriver: Send + Sync + 'static {
	/// The unique ID of the DMX driver
	fn get_id<'a>(&'a self) -> &'a str;

	/// The human-readable name of the DMX driver
	fn get_name<'a>(&'a self) -> &'a str;

	/// A human-readable description of the driver, such as what devices and protocols it uses
	fn get_description<'a>(&'a self) -> &'a str;

	/// Gets a form used by the UI for linking a universe to this driver
	async fn get_register_universe_form(&self) -> FormDescriptor;

	/// Registers a universe using data from a filled-in form
	async fn register_universe(
		&self,
		id: &Uuid,
		form: SerializedData,
	) -> Result<(), RegisterUniverseError>;

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
#[derive(Error)]
pub enum RegisterUniverseError {
	#[error("Could not parse form data when registering universe:\n{0}")]
	InvalidData(String),
	#[error("An error occurred in the transport driver while registering the universe:\n{0}")]
	Other(String),
}
impl_deserialize_err!(RegisterUniverseError, Self::InvalidData);

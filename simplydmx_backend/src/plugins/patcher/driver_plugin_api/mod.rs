use std::ops::Deref;

pub use crate::{impl_anyhow, utilities::forms::FormDescriptor};
use crate::{
	impl_deserialize_err, mixer_utils::state::FullMixerOutput,
	utilities::serialized_data::SerializedData,
};
use async_std::sync::{Arc, RwLockReadGuard};
pub use uuid::Uuid;

use async_trait::async_trait;
use simplydmx_plugin_framework::*;
use thiserror::Error;

use super::state::PatcherContext;
pub use super::PatcherInterface;
pub use super::{
	fixture_types::*,
	state::{FixtureInstance, SharablePatcherState},
};

pub struct SharableStateWrapper<'a>(RwLockReadGuard<'a, PatcherContext>);
impl<'a> SharableStateWrapper<'a> {
	pub fn new(lock_guard: RwLockReadGuard<'a, PatcherContext>) -> SharableStateWrapper<'a> {
		return SharableStateWrapper(lock_guard);
	}
}
impl<'a> Deref for SharableStateWrapper<'a> {
	type Target = SharablePatcherState;
	fn deref(&self) -> &Self::Target {
		return &self.0.sharable;
	}
}

#[async_trait]
pub trait OutputDriver: Send + Sync + 'static {
	/// Gets the ID of the output driver, for use internally
	fn get_id(&self) -> String;

	/// Gets the name of the output driver, for display in the UI
	fn get_name(&self) -> String;

	/// Gets a description of the output driver, for display in the UI
	fn get_description(&self) -> String;

	/// Imports a fixture description with the given SerializedData instance, as packaged in the bundle
	async fn import_fixture(&self, id: &Uuid, data: SerializedData) -> Result<(), ImportError>;

	/// Exports driver-specific information about the fixture for saving in a JSON format
	async fn export_fixture_json(&self, id: &Uuid) -> anyhow::Result<serde_json::Value>;

	/// Exports driver-specific information about the fixture for saving in a CBOR format
	async fn export_fixture_cbor(&self, id: &Uuid) -> anyhow::Result<Vec<u8>>;

	/// Gets a FormDescriptor to be sent to the UI for display to the user. The form descriptor should be
	/// detailed enough to allow the UI to generate a struct sufficient for use within `create_fixture_instance`.
	async fn get_creation_form(&self, fixture_info: &FixtureInfo) -> anyhow::Result<FormDescriptor>;

	/// Creates an instance of a fixture, based on data provided by the UI, which should have been derived from
	/// the form returned in `get_creation_form`.
	async fn create_fixture_instance(
		&self,
		patcher_state: &SharablePatcherState,
		fixture_id: &Uuid,
		fixture_type_info: &FixtureInfo,
		personality_id: &str,
		form: SerializedData,
	) -> Result<(), CreateInstanceError>;

	/// Removes an instance of a fixture.
	async fn remove_fixture_instance(&self, id: &Uuid) -> anyhow::Result<()>;

	/// Gets a copy of the edit form for the plugin in its current state
	async fn get_edit_form(&self, instance_id: &Uuid) -> anyhow::Result<FormDescriptor>;

	/// Edits an instance of a fixture based on data from the form returned in `get_edit_form`
	async fn edit_fixture_instance(
		&self,
		patcher_state: &SharablePatcherState,
		instance_id: &Uuid,
		fixture_type_info: &FixtureInfo,
		personality_id: &str,
		form: SerializedData,
	) -> Result<(), EditInstanceError>;

	/// Sends updates to the output.
	///
	/// This function's implementation must be fast and infallible. To resolve speed, try to keep information you would
	/// normally wait on in a cache or queue so it doesn't need to be fetched while rendering, and eliminate external
	/// I/O within reason.
	///
	/// This function must be infallible. No panicking should occur here. Ignore all errors (log them through the plugin
	/// framework if possible) and keep going. This is where the real-time data output happens and has huge consequences
	/// in a live setting.
	///
	/// `data` is the full mixer output. It is the responsibility of the driver to filter for items it cares about
	async fn send_updates<'a>(
		&self,
		patcher_data: &'a SharableStateWrapper<'a>,
		data: Arc<FullMixerOutput>,
	);
}

#[portable]
#[derive(Error)]
/// A generic error originating from an OutputDriver interface when importing a fixture definition
pub enum ImportError {
	#[error("Could not parse form data:\n{0}")]
	InvalidData(String),
	#[error("An error occurred while importing the fixture:\n{0}")]
	Other(String),
}
impl_deserialize_err!(ImportError, Self::InvalidData);
impl_anyhow!(ImportError, Self::Other);

#[portable]
#[derive(Error)]
/// A generic error originating from an OutputDriver interface when creating a fixture instance
pub enum CreateInstanceError {
	#[error("Could not parse form data:\n{0}")]
	InvalidData(String),
	#[error("An error occurred while creating the fixture:\n{0}")]
	Other(String),
}
impl_deserialize_err!(CreateInstanceError, Self::InvalidData);
impl_anyhow!(CreateInstanceError, Self::Other);

#[portable]
#[derive(Error)]
/// A generic error originating from an OutputDriver interface when editing an existing fixture instance
pub enum EditInstanceError {
	#[error("Could not parse form data:\n{0}")]
	InvalidData(String),
	#[error("An error occurred while editing the fixture:\n{0}")]
	Other(String),
}
impl_deserialize_err!(EditInstanceError, Self::InvalidData);
impl_anyhow!(EditInstanceError, Self::Other);

use std::ops::Deref;

use async_std::sync::{Arc, RwLockReadGuard};
pub use uuid::Uuid;
use crate::{
	impl_deserialize_err,
	utilities::serialized_data::SerializedData,
	mixer_utils::state::FullMixerOutput,
};
pub use crate::utilities::forms::FormDescriptor;

use async_trait::async_trait;
use simplydmx_plugin_framework::*;

pub use super::PatcherInterface;
use super::state::PatcherContext;
pub use super::{
	fixture_types::*,
	state::{
		SharablePatcherState,
		FixtureInstance,
	},
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
	async fn export_fixture_json(&self, id: &Uuid) -> Option<serde_json::Value>;

	/// Exports driver-specific information about the fixture for saving in a CBOR format
	async fn export_fixture_cbor(&self, id: &Uuid) -> Option<Vec<u8>>;

	/// Gets a FormDescriptor to be sent to the UI for display to the user. The form descriptor should be
	/// detailed enough to allow the UI to generate a struct sufficient for use within `create_fixture_instance`.
	async fn get_creation_form(&self) -> FormDescriptor;

	/// Creates an instance of a fixture, based on data provided by the UI, which should have been derived from
	/// the form returned in `get_creation_form`.
	async fn create_fixture_instance(&self, id: &Uuid, form: SerializedData) -> Result<(), CreateInstanceError>;

	/// Removes an instance of a fixture.
	async fn remove_fixture_instance(&self, id: &Uuid);

	/// Gets a copy of the edit form for the plugin in its current state
	async fn get_edit_form(&self, instance_id: &Uuid) -> FormDescriptor;

	/// Edits an instance of a fixture based on data from the form returned in `get_edit_form`
	async fn edit_fixture_instance(&self, id: &Uuid, form: SerializedData) -> Result<(), EditError>;

	/// Sends updates to the output.
	///
	/// `patcher_interface` serves purely as a means of accessing fixture data by calling
	/// `patcher_interface.get_sharable_state().await`. Use of this interface for any other purpose is undefined behavior
	/// and could result in a deadlock. This reference is not to be stored.
	///
	/// One solution to prevent slowing down the rest of the application if updates take too long is to implement a message
	/// queue that gets drained on each loop iteration, taking the most recent event, and push to that queue, returning
	/// immediately.
	///
	/// `data` is the full mixer output. It is the responsibility of the driver to filter for items it cares about
	async fn send_updates(&self, patcher_interface: PatcherInterface, data: Arc<FullMixerOutput>);

}

#[portable]
#[serde(tag = "type")]
pub enum ImportError {
	InvalidData,
	Other(String),
	Unknown,
}
impl_deserialize_err!(ImportError, Self::InvalidData);

#[portable]
#[serde(tag = "type")]
pub enum CreateInstanceError {
	InvalidData,
	Other(String),
	Unknown,
}
impl_deserialize_err!(CreateInstanceError, Self::InvalidData);

#[portable]
#[serde(tag = "type")]
pub enum EditError {
	InvalidData,
	Other(String),
	Unknown,
}
impl_deserialize_err!(EditError, Self::InvalidData);

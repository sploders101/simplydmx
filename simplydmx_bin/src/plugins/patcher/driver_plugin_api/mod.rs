pub use uuid::Uuid;
use crate::{impl_deserialize_err, utilities::serialized_data::SerializedData};
pub use crate::utilities::forms::FormDescriptor;

use async_trait::async_trait;
use simplydmx_plugin_framework::*;

#[async_trait]
pub trait OutputDriver: Send + Sync + 'static {

	// Metadata
	fn get_id(&self) -> String;
	fn get_name(&self) -> String;
	fn get_description(&self) -> String;

	// Fixture library imports/exports
	async fn import_fixture(&self, id: &Uuid, data: SerializedData) -> Result<(), ImportError>;
	async fn export_fixture_json(&self, id: &Uuid) -> Option<serde_json::Value>;
	async fn export_fixture_bincode(&self, id: &Uuid) -> Option<Vec<u8>>;

	// Fixture creation/removal
	async fn get_creation_form(&self) -> FormDescriptor;
	async fn create_fixture_instance(&self, id: &Uuid, form: SerializedData) -> Result<(), CreateInstanceError>;
	async fn remove_fixture_instance(&self, id: &Uuid);

	// Fixture editing
	async fn get_edit_form(&self) -> FormDescriptor;
	async fn edit_fixture_instance(&self, id: &Uuid, form: SerializedData) -> Result<(), EditError>;

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

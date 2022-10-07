use simplydmx_plugin_framework::*;
use uuid::Uuid;
use crate::utilities::serialized_data::SerializedData;

use super::{
	PatcherInterface,
	driver_plugin_api::{
		FixtureBundle,
		SharablePatcherState,
	},
	interface::{
		ImportFixtureError,
		CreateFixtureError,
	},
};

#[interpolate_service(
	"import_fixture",
	"Import fixture definition",
	"Import a fixture definition",
)]
impl ImportFixtureDefinition {

	#![inner_raw(PatcherInterface)]

	pub fn new(patcher_interface: PatcherInterface) -> Self { Self(patcher_interface) }

	#[service_main(
		("Fixture Bundle", "The fixture bundle you would like to import"),
		("Result", "Whether or not the import succeeded"),
	)]
	async fn main(self, fixture_bundle: FixtureBundle) -> Result<(), ImportFixtureError> {
		return self.0.import_fixture(fixture_bundle).await;
	}

}


#[interpolate_service(
	"create_fixture",
	"Create Fixture",
	"Creates a new fixture for use in the application",
)]
impl CreateFixture {

	#![inner_raw(PatcherInterface)]

	pub fn new(patcher_interface: PatcherInterface) -> Self { Self(patcher_interface) }

	#[service_main(
		("Fixture Type", "The UUID of the fixture description from the library that this new fixture should be based on"),
		("Personality", "The personality that this fixture should be based on."),
		("Name", "The optional user-specified name of the fixture"),
		("Comments", "Any additional comments the user would like to leave on this fixture instance"),
		("Form Data", "Data from the user in the format specified by the fixture controller's creation form"),
		("Result: Uuid", "Result containing the UUID of hte new fixture or an error"),
	)]
	async fn main(self, fixture_type: Uuid, personality: String, name: Option::<String>, comments: Option::<String>, form_data: SerializedData) -> Result<Uuid, CreateFixtureError> {
		return self.0.create_fixture(fixture_type, personality, name, comments, form_data).await;
	}

}

#[interpolate_service(
	"get_patcher_state",
	"Get Patcher State",
	"Retrieves the current state of the patcher, with libraries, registered fixtures, etc.",
)]
impl GetPatcherState {

	#![inner_raw(PatcherInterface)]

	pub fn new(patcher_interface: PatcherInterface) -> Self { Self(patcher_interface) }

	#[service_main(
		("Patcher State", "The sharable state of the patcher"),
	)]
	async fn main(self) -> SharablePatcherState {
		let state = self.0.get_sharable_state().await;
		return SharablePatcherState::clone(&state);
	}

}

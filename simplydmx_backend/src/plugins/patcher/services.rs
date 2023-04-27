use crate::utilities::{forms::FormDescriptor, serialized_data::SerializedData};
use simplydmx_plugin_framework::*;
use uuid::Uuid;

use super::{
	driver_plugin_api::{FixtureBundle, SharablePatcherState},
	interface::{
		CreateFixtureError, EditFixtureError, GetCreationFormError, GetEditFormError,
		ImportFixtureError,
	},
	PatcherInterface,
};

#[interpolate_service(
	"import_fixture",
	"Import fixture definition",
	"Import a fixture definition"
)]
impl ImportFixtureDefinition {
	#![inner_raw(PatcherInterface)]

	pub fn new(patcher_interface: PatcherInterface) -> Self {
		Self(patcher_interface)
	}

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
	"Creates a new fixture for use in the application"
)]
impl CreateFixture {
	#![inner_raw(PatcherInterface)]

	pub fn new(patcher_interface: PatcherInterface) -> Self {
		Self(patcher_interface)
	}

	#[service_main(
		("Fixture Type", "The UUID of the fixture description from the library that this new fixture should be based on"),
		("Personality", "The personality that this fixture should be based on."),
		("Name", "The optional user-specified name of the fixture"),
		("Comments", "Any additional comments the user would like to leave on this fixture instance"),
		("Form Data", "Data from the user in the format specified by the fixture controller's creation form"),
		("Result: Uuid", "Result containing the UUID of hte new fixture or an error"),
	)]
	async fn main(
		self,
		fixture_type: Uuid,
		personality: String,
		name: Option::<String>,
		comments: Option::<String>,
		form_data: SerializedData,
	) -> Result<Uuid, CreateFixtureError> {
		return self
			.0
			.create_fixture(fixture_type, personality, name, comments, form_data)
			.await;
	}
}

#[interpolate_service(
	"get_patcher_state",
	"Get Patcher State",
	"Retrieves the current state of the patcher, with libraries, registered fixtures, etc."
)]
impl GetPatcherState {
	#![inner_raw(PatcherInterface)]

	pub fn new(patcher_interface: PatcherInterface) -> Self {
		Self(patcher_interface)
	}

	#[service_main(
		("Patcher State", "The sharable state of the patcher"),
	)]
	async fn main(self) -> SharablePatcherState {
		let state = self.0.get_sharable_state().await;
		return SharablePatcherState::clone(&state);
	}
}

#[interpolate_service(
	"get_creation_form",
	"Get Creation Form",
	"Queries the given fixture's driver for a fixture creation form to display"
)]
impl GetCreationForm {
	#![inner_raw(PatcherInterface)]

	pub fn new(patcher_interface: PatcherInterface) -> Self {
		Self(patcher_interface)
	}

	#[service_main(
		("Fixture Type", "The UUID of the fixture within the fixture library that you would like to create an instance of", "fixture-type-uuid"),
		("Form Descriptor", "A dynamic form descriptor that can be used to build visual elements for the user to input the required data"),
	)]
	async fn main(self, fixture_type: Uuid) -> Result<FormDescriptor, GetCreationFormError> {
		return self.0.get_creation_form(&fixture_type).await;
	}
}

#[interpolate_service(
	"get_edit_form",
	"Get Edit Form",
	"Queries the given fixture's driver for a fixture edit form to display"
)]
impl GetEditForm {
	#![inner_raw(PatcherInterface)]

	pub fn new(patcher_interface: PatcherInterface) -> Self {
		Self(patcher_interface)
	}

	#[service_main(
		("Fixture ID", "The UUID of the particular fixture instance you would like to edit"),
		("Form Descriptor", "A result containing either a FormDescriptor object or an error"),
	)]
	async fn main(self, fixture_id: Uuid) -> Result<FormDescriptor, GetEditFormError> {
		return self.0.get_edit_form(&fixture_id).await;
	}
}

#[interpolate_service(
	"edit_fixture",
	"Edit Fixture",
	"Edits the requested fixture using data provided by the user"
)]
impl EditFixture {
	#![inner_raw(PatcherInterface)]

	pub fn new(patcher_interface: PatcherInterface) -> Self {
		Self(patcher_interface)
	}

	#[service_main(
		("Fixture Instance ID", "The UUID of the particular fixture instance you would like to edit"),
		("Personality ID", "The UUID of the personality you would like to use for the fixture"),
		("Name", "An arbitrary name"),
		("Comments", "Arbitrary comments"),
		("Form Data", "Form data as given by the dynamic form from `get_edit_form`"),
		("Form Descriptor", "A result containing either a FormDescriptor object or an error"),
	)]
	async fn main(
		self,
		instance_id: Uuid,
		personality: String,
		name: Option::<String>,
		comments: Option::<String>,
		form_data: SerializedData,
	) -> Result<(), EditFixtureError> {
		return self.0.edit_fixture(&instance_id, personality, name, comments, form_data).await;
	}
}

#[interpolate_service(
	"edit_fixture_placement",
	"Edit Fixture Placement",
	"Edits the x,y coordinates of the fixture within the visualizer"
)]
impl EditFixturePlacement {
	#![inner_raw(PatcherInterface)]

	pub fn new(patcher_interface: PatcherInterface) -> Self {
		Self(patcher_interface)
	}

	#[service_main(
		("Fixture ID", "The UUID of the particular fixture instance you would like to edit"),
		("X Coordinate", "The fixture's new X coordinate"),
		("Y Coordinate", "The fixture's new Y coordinate")
	)]
	async fn main(self, fixture_id: Uuid, x: u16, y: u16) {
		return self.0.edit_fixture_placement(&fixture_id, x, y).await;
	}
}

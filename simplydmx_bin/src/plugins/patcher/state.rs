use std::collections::HashMap;
use uuid::Uuid;

use super::fixture_types::FixtureInfo;

use simplydmx_plugin_framework::*;

#[portable]
pub struct PatcherContext {
	pub library: HashMap<Uuid, FixtureInfo>,
	pub fixture_order: Vec<Uuid>,
	pub fixtures: HashMap<Uuid, FixtureInstance>,
}

impl PatcherContext {
	pub fn new() -> Self {
		return PatcherContext {
			library: HashMap::new(),
			fixture_order: Vec::new(),
			fixtures: HashMap::new(),
		};
	}
}


/// Identifies an individual instance of a fixture
#[portable]
pub struct FixtureInstance {

	/// The ID of this particular fixture
	pub id: Uuid,

	/// The ID of this fixture's type
	pub fixture_id: Uuid,

	/// The personality identifier of this fixture
	pub personality: String,

	/// An arbitrary name for this particular instance of the fixture
	pub name: Option<String>,

	/// Arbitrary comments about this particular instance left by the user
	pub comments: Option<String>,

}

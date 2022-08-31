use std::collections::HashMap;
use uuid::Uuid;

use super::types::FixtureInfo;

use simplydmx_plugin_framework::*;

#[portable]
pub struct PatcherContext {
	pub library: HashMap<Uuid, FixtureInfo>,
	pub fixtures: HashMap<Uuid, FixtureInstance>,
}

impl PatcherContext {
	pub fn new() -> Self {
		return PatcherContext {
			library: HashMap::new(),
			fixtures: HashMap::new(),
		};
	}
}


#[portable]
pub struct FixtureInstance {
	/// The ID of this particular fixture
	pub id: Uuid,

	/// The ID of this fixture's type
	pub fixture_id: Uuid,

	/// The personality identifier of this fixture
	pub personality: String,

	pub name: Option<String>,
	pub comments: Option<String>,
}

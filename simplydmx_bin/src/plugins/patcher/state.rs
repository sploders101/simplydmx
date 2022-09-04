use std::collections::HashMap;
use uuid::Uuid;

use super::{fixture_types::FixtureInfo, driver_plugin_api::OutputDriver};

use simplydmx_plugin_framework::*;

pub struct PatcherContext {
	pub output_drivers: HashMap<String, Box<dyn OutputDriver>>,
	pub sharable: SharablePatcherState,
}
impl PatcherContext {
	pub fn new() -> Self {
		return PatcherContext {
			output_drivers: HashMap::new(),
			sharable: SharablePatcherState {
				library: HashMap::new(),
				fixture_order: Vec::new(),
				fixtures: HashMap::new(),
			},
		}
	}
}


#[portable]
pub struct SharablePatcherState {
	pub library: HashMap<Uuid, FixtureInfo>,
	pub fixture_order: Vec<Uuid>,
	pub fixtures: HashMap<Uuid, FixtureInstance>,
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

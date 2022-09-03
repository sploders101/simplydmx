use std::collections::HashMap;
use uuid::Uuid;

use simplydmx_plugin_framework::*;

use super::{
	fixture_types::DMXFixtureData,
	driver_types::DMXDriverDescriptor,
};

#[portable]
pub struct DMXState {
	pub output_types: HashMap<String, DMXDriverDescriptor>,
	pub library: HashMap<Uuid, DMXFixtureData>,
	pub fixtures: HashMap<Uuid, FixtureInstance>,
	pub universes: HashMap<Uuid, UniverseInstance>,
}
impl DMXState {
	pub fn new() -> Self {
		return DMXState {
			output_types: HashMap::new(),
			library: HashMap::new(),
			fixtures: HashMap::new(),
			universes: HashMap::new(),
		}
	}
}

#[portable]
pub struct FixtureInstance {
	pub universe: Uuid,
	pub offset: u16,
}

#[portable]
pub struct UniverseInstance {
	pub id: Uuid,
	pub controller: Option<UniverseController>,
}

#[portable]
pub struct UniverseController {}

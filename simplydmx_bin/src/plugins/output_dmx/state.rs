use std::{
	collections::HashMap,
	sync::Arc,
};
use uuid::Uuid;

use simplydmx_plugin_framework::*;

use super::{
	fixture_types::DMXFixtureData,
	driver_types::DMXDriver,
};

pub struct DMXState {
	pub drivers: HashMap<String, Arc<Box<dyn DMXDriver>>>,
	pub library: HashMap<Uuid, DMXFixtureData>,
	pub fixtures: HashMap<Uuid, DMXFixtureInstance>,
	pub universes: HashMap<Uuid, UniverseInstance>,
}
impl DMXState {
	pub fn new() -> Self {
		return DMXState {
			drivers: HashMap::new(),
			library: HashMap::new(),
			fixtures: HashMap::new(),
			universes: HashMap::new(),
		}
	}
}

#[portable]
pub struct DMXFixtureInstance {
	pub universe: Option<Uuid>,
	pub offset: Option<u16>,
}

#[portable]
pub struct UniverseInstance {
	pub id: Uuid,
	pub controller: Option<String>,
}

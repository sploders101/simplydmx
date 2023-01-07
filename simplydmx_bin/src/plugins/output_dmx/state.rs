use std::{
	collections::HashMap,
	sync::Arc,
};
use serde::{
	Serialize,
	Deserialize,
};
use uuid::Uuid;

use simplydmx_plugin_framework::*;

use super::{
	fixture_types::DMXFixtureData,
	driver_types::DMXDriver,
	interface::DMXShowSave,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct DMXState {
	#[serde(skip)]
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
	pub fn from_file(file: DMXShowSave) -> Self {
		return DMXState {
			drivers: HashMap::new(),
			library: file.library,
			fixtures: file.fixtures,
			universes: file.universes,
		}
	}
}

#[portable]
/// This holds DMX-specific information about a fixture instance
pub struct DMXFixtureInstance {
	pub universe: Option<Uuid>,
	pub offset: Option<u16>,
}

#[portable]
/// This represents a DMX universe instance
pub struct UniverseInstance {
	pub id: Uuid,
	pub name: String,
	pub controller: Option<String>,
}

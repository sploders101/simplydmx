use std::collections::HashMap;

use simplydmx_plugin_framework::*;

#[portable]
pub struct DMXFixtureData {
	pub personalities: HashMap<String, DMXPersonalityData>,
}

#[portable]
pub struct DMXPersonalityData {
	pub dmx_channel_order: Vec<String>,
}

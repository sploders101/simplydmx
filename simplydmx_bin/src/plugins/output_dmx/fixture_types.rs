use std::collections::HashMap;

use simplydmx_plugin_framework::*;

#[portable]
/// DMX-specific components of a fixture definition.
///
/// This goes in the output_info, property of a `FixtureBundle` object
pub struct DMXFixtureData {
	pub personalities: HashMap<String, DMXPersonalityData>,
}

#[portable]
/// DMX-specific personality data
///
/// This goes inside a `DMXFixtureData` instance
pub struct DMXPersonalityData {
	pub dmx_channel_order: Vec<String>,
}

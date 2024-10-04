use layering::{Layer, Submaster};
use patcher::{fixture_types::FixtureProfile, DmxFixtureInstance};
use utils::{id_alloc::Id, smallmap::SmallMap};
use uuid::Uuid;

mod layering;
mod patcher;
mod utils;

pub struct SimplyDmx {
	/// Library of fixture profiles usable from the patcher
	dmx_fixture_library: SmallMap<Id, FixtureProfile>,
	/// Fixtures registered in the show file
	dmx_fixtures: SmallMap<Id, DmxFixtureInstance>,
	/// Submasters available for blending via layers
	submasters: SmallMap<Id, Submaster>,
	/// Layers used in the final render
	layers: SmallMap<Id, Layer>,
}

impl SimplyDmx {
	pub fn render(&mut self) {
		
	}
}

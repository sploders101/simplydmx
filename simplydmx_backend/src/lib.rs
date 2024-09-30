use patcher::fixture_types::FixtureProfile;
use rustc_hash::FxHashMap;
use utils::id_alloc::Id;
use uuid::Uuid;

mod layering;
mod patcher;
mod utils;

pub struct SimplyDmx {
	fixture_library: FxHashMap<Uuid, FixtureProfile>,
	layers: FxHashMap<Id, Layer>,
}

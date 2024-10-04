use crate::utils::id_alloc::Id;

pub mod fixture_types;

/// Represents a single instance of a fixture
pub struct DmxFixtureInstance {
	/// The ID of the fixture profile from the library
	fixture_profile: Id,
	/// The ID of the logical universe the fixture is in
	universe: Id,
	/// The offset of the fixture within the universe
	offset: u16,
}

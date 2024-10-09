use crate::utils::id_alloc::Id;

pub mod fixture_types;

/// Represents a single instance of a fixture
pub struct DmxFixtureInstance {
	/// The ID of the fixture profile from the library
	pub fixture_profile: Id,
	/// The "personality" or "mode" that the fixture is set to
	pub personality: u32,
	/// The ID of the logical universe the fixture is in
	pub universe: Id,
	/// The offset of the fixture within the universe
	pub offset: u16,
}

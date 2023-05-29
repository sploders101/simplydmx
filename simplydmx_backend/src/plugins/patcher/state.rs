use super::{driver_plugin_api::OutputDriver, fixture_types::FixtureInfo};
use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;
use std::sync::Arc;
use uuid::Uuid;

pub struct PatcherContext {
	pub output_drivers: FxHashMap<String, Arc<Box<dyn OutputDriver>>>,
	pub sharable: SharablePatcherState,
}
impl PatcherContext {
	pub fn new() -> Self {
		return PatcherContext {
			output_drivers: FxHashMap::default(),
			sharable: SharablePatcherState {
				library: FxHashMap::default(),
				fixture_order: Vec::default(),
				fixtures: FxHashMap::default(),
			},
		};
	}
	pub fn from_file(file: SharablePatcherState) -> Self {
		return PatcherContext {
			output_drivers: FxHashMap::default(),
			sharable: file,
		};
	}
}

#[portable]
/// Sharable (and serializable) component of the patcher state containing
/// information about registered fixtures
pub struct SharablePatcherState {
	pub library: FxHashMap<Uuid, FixtureInfo>,
	pub fixture_order: Vec<Uuid>,
	pub fixtures: FxHashMap<Uuid, FixtureInstance>,
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

	/// Information about this particular fixture for the visualizer
	pub visualization_info: VisualizationInfo,
}

#[portable]
pub struct VisualizationInfo {
	pub x: u16,
	pub y: u16,
}
impl Default for VisualizationInfo {
	fn default() -> Self {
		return VisualizationInfo {
			x: 0,
			y: 0,
		};
	}
}

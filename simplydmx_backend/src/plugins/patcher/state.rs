use async_std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

use super::{driver_plugin_api::OutputDriver, fixture_types::FixtureInfo};

use simplydmx_plugin_framework::*;

pub struct PatcherContext {
	pub output_drivers: HashMap<String, Arc<Box<dyn OutputDriver>>>,
	pub sharable: SharablePatcherState,
}
impl PatcherContext {
	pub fn new() -> Self {
		return PatcherContext {
			output_drivers: HashMap::new(),
			sharable: SharablePatcherState {
				library: HashMap::new(),
				fixture_order: Vec::new(),
				fixtures: HashMap::new(),
			},
		};
	}
	pub fn from_file(file: SharablePatcherState) -> Self {
		return PatcherContext {
			output_drivers: HashMap::new(),
			sharable: file,
		};
	}
}

#[portable]
/// Sharable (and serializable) component of the patcher state containing
/// information about registered fixtures
pub struct SharablePatcherState {
	pub library: HashMap<Uuid, FixtureInfo>,
	pub fixture_order: Vec<Uuid>,
	pub fixtures: HashMap<Uuid, FixtureInstance>,
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

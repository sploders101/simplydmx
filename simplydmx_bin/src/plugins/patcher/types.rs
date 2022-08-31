use simplydmx_plugin_framework::*;
use uuid::Uuid;
use crate::plugins::mixer::exported_types::SnapData;
use std::collections::HashMap;

use crate::plugins::mixer::exported_types::BlendingScheme;

#[portable]
pub struct FixtureBundle {
	pub id: Uuid,
	pub fixture_info: FixtureInfo,
	pub controller: (String, String),
	pub output_info: SerializedData,
}

#[portable]
#[serde(untagged)]
pub enum SerializedData {
	JSON(serde_json::Value),
	Bincode(Vec<u8>),
}

#[portable]
pub struct FixtureInfo {
	pub name: String,
	pub short_name: Option<String>,

	pub metadata: FixtureMeta,

	pub channels: HashMap<String, Channel>,
	pub personalities: HashMap<String, Personality>,
	pub output_info: OutputInfo,
}

/// Plugin-specific output info
#[portable]
pub struct OutputInfo {
	pub plugin_id: String,
	pub update: String,
	pub export_json: String,
	pub export_bincode: String,
}

#[portable]
pub struct FixtureMeta {
	pub manufacturer: Option<String>,
	pub manual_link: Option<String>,
}

#[portable]
pub struct Channel {
	pub size: ChannelSize,
	#[serde(default)]
	pub default: u16,
	pub ch_type: ChannelType,
}

#[portable]
pub enum ChannelSize {
	U8,
	U16,
}

#[portable]
#[serde(tag = "type")]
pub enum ChannelType {
	Segmented {
		segments: Vec<Segment>,
		priority: BlendingScheme,
		snapping: Option<SnapData>,
	},
	Linear {
		priority: BlendingScheme,
	},
}

#[portable]
/// Identifies a segment used in a segmented channel
pub struct Segment {
	pub start: u16,
	pub end: u16,
	pub name: String,
	pub id: String,
}

#[portable]
/// Identifies non-implementation-specific features of a personality.
///
/// Implementation-specific features of a personality such as channel order should
/// should be stored in the output data for use by the output plugin.
pub struct Personality {
	/// A vector of channel IDs used in the personality
	pub available_channels: Vec<String>,
}

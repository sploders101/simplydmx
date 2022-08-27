use simplydmx_plugin_framework::*;
use crate::plugins::mixer::exported_types::SnapData;
use std::collections::HashMap;

use crate::plugins::mixer::exported_types::BlendingScheme;

#[portable]
pub struct FixtureInfo {
	name: String,
	short_name: Option<String>,

	metadata: FixtureMeta,

	channels: HashMap<String, Channel>,
	personalities: HashMap<String, Personality>,
	output_info: OutputInfo,
}

#[portable]
pub enum OutputInfo {
	DMX(String)
}

#[portable]
pub struct FixtureMeta {
	manufacturer: Option<String>,
	manual_link: Option<String>,
}

#[portable]
pub struct Channel {
	size: ChannelSize,
	ch_type: ChannelType,
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
	start: u16,
	end: u16,
	name: String,
	id: String,
}

#[portable]
/// Identifies non-implementation-specific features of a personality.
///
/// Implementation-specific features of a personality such as channel order should
/// should be stored in the output data for use by the output plugin.
pub struct Personality {
	/// A vector of channel IDs used in the personality
	available_channels: Vec<String>,
}

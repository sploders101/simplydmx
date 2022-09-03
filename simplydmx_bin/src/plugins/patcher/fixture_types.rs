use simplydmx_plugin_framework::*;
use uuid::Uuid;
use crate::plugins::mixer::exported_types::SnapData;
use std::collections::HashMap;

use crate::plugins::mixer::exported_types::BlendingScheme;

#[portable]
pub struct FixtureBundle {

	/// Contains information about the fixture used for blending, identification, and capability detection
	pub fixture_info: FixtureInfo,

	/// Service to call with arguments: (Uuid, CustomDataType).
	///
	/// The service's JSON or Bincode RPC API will be invoked depending on bundle serialization method
	pub controller: (String, String),

	/// Stores output information for the controller
	pub output_info: SerializedData,

}

/// Data type used to hold a serialized instance of an arbitrary data type.
///
/// This is intended to encapsulate dynamically-typed data intended for deserialization by the output plugin
#[portable]
#[serde(tag = "t", content = "c")]
pub enum SerializedData {
	Bincode(Vec<u8>),
	JSON(serde_json::Value),
}

/// Data type that contains generic, protocol-erased information about a fixture such as name,
/// metadata, personalities, and references to services within the output controller.
#[portable]
pub struct FixtureInfo {

	/// The UUID to store this fixture as. This should be regenerated whenever a breaking change is made to the data.
	///
	/// Instances of this fixture will contain this UUID as a reference to the source data.
	pub id: Uuid,

	/// The human-readable name of the fixture
	pub name: String,

	/// The name to use on displays that require a shorter variant
	pub short_name: Option<String>,

	/// The manufacturer of the light, used for grouping and display
	pub manufacturer: Option<String>,

	/// The family of the light, used for grouping and display
	pub family: Option<String>,

	/// Metadata about the feature that would only be useful to the user
	pub metadata: FixtureMeta,

	/// Pool of channels for a personality to choose from
	pub channels: HashMap<String, Channel>,

	/// Personalities, or modes, available on a light. They can contain alternative channel layouts.
	pub personalities: HashMap<String, Personality>,

	///
	pub output_info: OutputInfo,

}

/// Stores information about the output controller
#[portable]
pub struct OutputInfo {

	/// The plugin ID of the output controller
	pub plugin_id: String,

	/// The name of the service to be called for exporting protocol-specific fixture data
	pub exporter: String,

}

/// Metadata about the fixture, used for display in the UI
#[portable]
pub struct FixtureMeta {
	pub manufacturer: Option<String>,
	pub manual_link: Option<String>,
}

/// Information about a specific channel available on the fixture
#[portable]
pub struct Channel {

	/// Size of the channel. SimplyDMX can store values as larger types, but the mixer will ensure the bounds of this
	/// type are met, and outputs will truncate data to this length
	pub size: ChannelSize,

	/// The default value, to be used in the background layer during blending
	#[serde(default)]
	pub default: u16,

	/// Dictates how the channel should be blended/controlled
	pub ch_type: ChannelType,

}

/// Dictates the size of the output. Values will be stored as the largest of these options, but bounds
/// will be enforced by the UI, mixer, and output will be truncated.
#[portable]
pub enum ChannelSize {
	U8,
	U16,
}

/// Describes information used for controlling and blending the channel
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

/// Identifies a segment used in a segmented channel
#[portable]
pub struct Segment {

	/// The minimum value available within this segment
	pub start: u16,

	/// The maximum value available within this segment
	pub end: u16,

	/// The name of the segment, for display in user interfaces
	pub name: String,

	/// An arbitrary ID used to identify this segment
	pub id: String,

}

/// Identifies non-implementation-specific features of a personality.
///
/// Implementation-specific features of a personality such as channel order should
/// should be stored in the output data for use by the output plugin.
#[portable]
pub struct Personality {
	/// A vector of channel IDs used in the personality
	pub available_channels: Vec<String>,
}

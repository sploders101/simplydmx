use crate::{mixer_utils::state::SnapData, utilities::serialized_data::SerializedData};
use simplydmx_plugin_framework::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::mixer_utils::state::BlendingScheme;

#[portable]
/// Importable fixture descriptor bundle that includes output driver-specific information
pub struct FixtureBundle {
	/// Contains information about the fixture used for blending, identification, and capability detection
	pub fixture_info: FixtureInfo,

	/// Stores output information for the controller
	pub output_info: SerializedData,
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

	/// Contains a string referencing the output driver associated with the fixture.
	pub output_driver: String,

	/// Contains data about groups of channels that can be assigned to a user-friendly controller
	///
	/// These get filtered by what channels are available in the selected personality
	pub control_groups: Vec<ControlGroup>,
}

/// Contains data about a group of channels that can be controlled using a special controller
#[portable]
pub enum ControlGroup {
	Intensity(String),
	RGBGroup { red: String, green: String, blue: String },
	CMYKGroup { cyan: String, magenta: String, yellow: String, black: String },
	PanTilt { pan: String, tilt: String },
	Gobo(String),
	ColorWheel(String),
	Zoom(String),
	GenericInput(String),
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
	/// Designates this channel as a virtual intensity channel.
	///
	/// If `None`, this channel is output directly.
	///
	/// If `Some(vec!["channel_id_1", ...])`, each channel listed will
	/// be inhibited by the value of this channel before being sent to
	/// the output driver.
	/// TODO: Implement this in the mixer
	pub intensity_emulation: Option<Vec<String>>,

	/// Size of the channel. SimplyDMX can store values as larger types,
	/// but the mixer will ensure the bounds of this type are met, and
	/// outputs will truncate data to this length
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

/// Describes an image to display
#[portable]
pub enum AssetDescriptor {
	BuiltIn(String),
	SVGInline(String),
}

/// Describes how a segment should be displayed to the user in the UI
#[portable]
pub enum SegmentDisplay {
	/// Displays
	Gobo { asset: AssetDescriptor },
	Color { red: u8, green: u8, blue: u8 },
	Image { asset: AssetDescriptor },
	Other,
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

	/// Indicates how the segment should be displayed to the user
	pub display: SegmentDisplay,
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

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use uuid::Uuid;

/// Data type that contains generic, protocol-erased information about a fixture such as name,
/// metadata, personalities, and references to services within the output controller.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixtureProfile {
	/// The UUID to store this fixture as. This should be regenerated whenever a breaking change is made to the data.
	///
	/// Instances of this fixture will contain this UUID as a reference to the source data.
	pub id: Uuid,

	/// The human-readable name of the fixture
	pub name: SmartString<LazyCompact>,

	/// The name to use on displays that require a shorter variant
	pub short_name: Option<SmartString<LazyCompact>>,

	/// The manufacturer of the light, used for grouping and display
	pub manufacturer: Option<SmartString<LazyCompact>>,

	/// The family of the light, used for grouping and display
	pub family: Option<SmartString<LazyCompact>>,

	/// A link to the manual
	pub manual: Option<SmartString<LazyCompact>>,

	/// Any additional notes that might be useful
	pub notes: Option<SmartString<LazyCompact>>,

	/// Pool of channels for a personality to choose from
	pub channels: FxHashMap<u32, Channel>,

	/// Personalities, or modes, available on a light. They can contain alternative channel layouts.
	pub personalities: FxHashMap<u32, Personality>,

	/// Information to be used in the serialization of fixture data
	// pub driver_info: FixtureDriverDetails,

	/// Contains data about groups of channels that can be assigned to a user-friendly controller
	///
	/// These get filtered by what channels are available in the selected personality
	pub control_groups: Vec<ControlGroup>,
}

/// Contains data about a group of channels that can be controlled using a special controller
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ControlGroup {
	/// A name can be specified for a non-standard control.
	///
	/// If a name is specified, the control will only be grouped with those from instances of
	/// identical fixtures.
	name: Option<SmartString<LazyCompact>>,

	/// Specifies the type of control to use and the associated channels
	channels: ControlGroupData,
}

/// Specifies the type of ControlGroup in use and associated channels
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControlGroupData {
	Intensity(u32),
	RGBGroup {
		red: u32,
		green: u32,
		blue: u32,
	},
	CMYKGroup {
		cyan: u32,
		magenta: u32,
		yellow: u32,
		black: u32,
	},
	PanTilt {
		pan: u32,
		tilt: u32,
	},
	Gobo(u32),
	ColorWheel(u32),
	Zoom(u32),
	GenericInput(u32),
}

/// Information about a specific channel available on the fixture
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Channel {
	/// Name of the channel
	pub name: SmartString<LazyCompact>,

	/// Designates this channel as a virtual intensity channel.
	///
	/// If `None`, this channel is output directly.
	///
	/// If `Some(vec!["channel_id_1", ...])`, each channel listed will
	/// be inhibited by the value of this channel before being sent to
	/// the output driver.
	pub intensity_emulation: Option<Vec<u32>>,

	/// Size of the channel. SimplyDMX can store values as larger types,
	/// but the mixer will ensure the bounds of this type are met, and
	/// outputs will truncate data to this length
	pub size: ChannelSize,

	/// The default value, to be used in the background layer during blending
	#[serde(default)]
	pub default: ChannelValue,

	/// Dictates how the channel should be blended/controlled
	pub ch_type: ChannelType,
}

/// This represents a vector that can scale automatically to different
/// integer sizes as necessary for the purpose of representing a channel
/// value.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ChannelValue {
	L8(u8),
	L16(u16),
}
impl Default for ChannelValue {
	fn default() -> Self {
		Self::L8(0)
	}
}
impl Eq for ChannelValue {}
impl PartialEq for ChannelValue {
	fn eq(&self, other: &Self) -> bool {
		matches!(self.cmp(other), std::cmp::Ordering::Equal)
	}
}
impl PartialOrd for ChannelValue {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		return Some(self.cmp(other));
	}
}
impl Ord for ChannelValue {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match (self, other) {
			(Self::L8(left), Self::L8(right)) => left.cmp(right),
			(Self::L8(left), Self::L16(right)) => Into::<u16>::into(*left).cmp(right),
			(Self::L16(left), Self::L16(right)) => left.cmp(right),
			(Self::L16(left), Self::L8(right)) => left.cmp(&Into::<u16>::into(*right)),
		}
	}
}
impl Into<u8> for ChannelValue {
	fn into(self) -> u8 {
		match self {
			Self::L8(value) => value,
			Self::L16(value) => (value / 257) as u8,
		}
	}
}
impl Into<u16> for ChannelValue {
	fn into(self) -> u16 {
		match self {
			Self::L8(value) => (value as u16) * 257,
			Self::L16(value) => value,
		}
	}
}

/// This represents a vector that can scale automatically to different
/// integer sizes as necessary for the purpose of representing an offset
/// from a base value during blending.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ChannelOffsetValue {
	L8(i16),
	L16(i32),
}
impl Default for ChannelOffsetValue {
	fn default() -> Self {
		Self::L8(0)
	}
}
impl Eq for ChannelOffsetValue {}
impl PartialEq for ChannelOffsetValue {
	fn eq(&self, other: &Self) -> bool {
		matches!(self.cmp(other), std::cmp::Ordering::Equal)
	}
}
impl PartialOrd for ChannelOffsetValue {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		return Some(self.cmp(other));
	}
}
impl Ord for ChannelOffsetValue {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match (self, other) {
			(Self::L8(left), Self::L8(right)) => left.cmp(right),
			(Self::L8(left), Self::L16(right)) => Into::<i32>::into(*left).cmp(right),
			(Self::L16(left), Self::L16(right)) => left.cmp(right),
			(Self::L16(left), Self::L8(right)) => left.cmp(&Into::<i32>::into(*right)),
		}
	}
}
impl Into<i16> for ChannelOffsetValue {
	fn into(self) -> i16 {
		match self {
			Self::L8(value) => value,
			Self::L16(value) => (value / 257) as i16,
		}
	}
}
impl Into<i32> for ChannelOffsetValue {
	fn into(self) -> i32 {
		match self {
			Self::L8(value) => (value as i32) * 257,
			Self::L16(value) => value,
		}
	}
}

/// Dictates the size of the output. Values will be stored as the largest of these options, but bounds
/// will be enforced by the UI, mixer, and output will be truncated.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ChannelSize {
	U8,
	U16,
}

/// Describes information used for controlling and blending the channel
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ChannelType {
	Segmented {
		segments: Vec<Segment>,
		priority: BlendingScheme,
		snapping: SnapData,
	},
	Linear {
		priority: BlendingScheme,
	},
}

/// Describes an image to display
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AssetDescriptor {
	BuiltIn(SmartString<LazyCompact>),
	SVGInline(SmartString<LazyCompact>),
}

/// Describes how a segment should be displayed to the user in the UI
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SegmentDisplay {
	/// Displays
	Gobo {
		asset: AssetDescriptor,
	},
	Color {
		red: u8,
		green: u8,
		blue: u8,
	},
	Image {
		asset: AssetDescriptor,
	},
	Other,
}

/// Identifies a segment used in a segmented channel
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Segment {
	/// The minimum value available within this segment
	pub start: ChannelValue,

	/// The maximum value available within this segment
	pub end: ChannelValue,

	/// The name of the segment, for display in user interfaces
	pub name: SmartString<LazyCompact>,

	/// Indicates how the segment should be displayed to the user
	pub display: SegmentDisplay,
}

/// Identifies non-implementation-specific features of a personality.
///
/// Implementation-specific features of a personality such as channel order should
/// should be stored in the output data for use by the output plugin.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Personality {
	/// Name of the personality
	pub name: SmartString<LazyCompact>,

	/// A vector of channel IDs used in the personality (references `FixtureProfile::channels` keys)
	pub available_channels: Vec<u32>,
}

/// The method in which conflicts are resolved while blending
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlendingScheme {
	/// Highest Takes Priority (usually used for intensity).
	///
	/// This takes the highest value, applying opacity to individual values on a backdrop value of
	/// 0 before deciding on a value to use.
	HTP,

	/// Latest Takes Priority (usually used for color).
	///
	/// This applies values sequentially in layer order while factoring in opacity, using the current
	/// running value as the backdrop.
	///
	/// This provides a smooth transition that's often used for color blending.
	LTP,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// This indicates if a value should snap to a new value. This is useful for things like gobos, where
/// intermediate values don't blend, and can instead cause distraction by rapidly switching between noticably
/// discrete states.
pub enum SnapData {
	/// Do not snap values. Output without transforming
	NoSnap,

	/// Take the latest value if the opacity is beyond the specified threshold
	SnapAt(ChannelValue),
}

use std::collections::HashMap;
use simplydmx_plugin_framework::*;
use uuid::Uuid;

/// Represents the data within a submaster used for blending
pub type SubmasterData = HashMap<Uuid, AbstractLayerLight>;

/// Represents the abstract data for a single light in a layer.
/// A value's binary may be masked if the output is u8 (integer overflow cast)
pub type AbstractLayerLight = HashMap<String, BlenderValue>;

/// Value to be used in a submaster with instructions for mixing it into the result
#[portable]
#[serde(tag = "type", content = "value")]
pub enum BlenderValue {

	/// Transparent value
	None,

	/// Static value, meaning at 100%, the channel should be exactly this value
	Static(u16),

	/// Offset value, meaning add this to the current calculated value during blending.
	/// In the event of an overflow, the max or min value will be used depending on the operation.
	Offset(i32),

}

/// Represents the full output of the mixer, ready to send out to the lights.
///
/// `Fixture ID --> Attribute ID --> Value`
pub type FullMixerOutput = HashMap<Uuid, FixtureMixerOutput>;

/// Represents the finished, pre-mixed values of a light.
///
/// `Attribute ID --> Value`
pub type FixtureMixerOutput = HashMap<String, u16>;

#[portable]
/// The method in which conflicts are resolved while blending
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

#[portable]
#[serde(tag = "type", content = "data")]
pub enum SnapData {

	/// Do not snap values. Output without transforming
	NoSnap,

	/// Take the latest value if the opacity is beyond the specified threshold
	SnapAt(u16),

}

#[portable]
pub struct BlendingData {

	/// Indicates how the value should be blended
	pub scheme: BlendingScheme,

	/// Indicates if value blending is allowed on the attribute
	pub snap: SnapData,

	/// Indicates whether or not overflows should wrap or stick to the outer bounds when blending offsets
	pub allow_wrap: bool,

	/// Specifies the maximum value for the attribute
	pub max_value: u16,

	/// Specifies the minimum value for the attribute
	pub min_value: u16,

}

pub type FullMixerBlendingData = HashMap<Uuid, HashMap<String, BlendingData>>;

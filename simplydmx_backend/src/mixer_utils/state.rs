use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;
use uuid::Uuid;

/// Represents the data within a submaster used for blending
#[portable]
pub type SubmasterData = FxHashMap<Uuid, AbstractLayerLight>;

/// Represents the abstract data for a single light in a layer.
/// A value's binary may be masked if the output is u8 (integer overflow cast)
#[portable]
pub type AbstractLayerLight = FxHashMap<String, BlenderValue>;

/// Value to be used in a submaster with instructions for mixing it into the result
#[portable]
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
#[portable]
pub type FullMixerOutput = FxHashMap<Uuid, FixtureMixerOutput>;

/// Represents the finished, pre-mixed values of a light.
///
/// `Attribute ID --> Value`
#[portable]
pub type FixtureMixerOutput = FxHashMap<String, u16>;

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
/// This indicates if a value should snap to a new value. This is useful for things like gobos, where
/// intermediate values don't blend, and can instead cause distraction by rapidly switching between noticably
/// discrete states.
pub enum SnapData {

	/// Do not snap values. Output without transforming
	NoSnap,

	/// Take the latest value if the opacity is beyond the specified threshold
	SnapAt(u16),

}

#[portable]
/// This contains data that indicates how a channel should be blended.
///
/// It is provided by the fixture description to tweak the properties of a layer's blending function.
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

pub type FullMixerBlendingData = FxHashMap<Uuid, FxHashMap<String, BlendingData>>;

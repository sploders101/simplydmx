use std::{
	collections::HashMap,
	sync::Arc,
};
use simplydmx_plugin_framework::*;
use uuid::Uuid;

// Use this for upgrades: https://serde.rs/attr-default.html

#[portable]
pub struct MixerContext {
	/// The layer bin any layer opacity changes should be written to.
	pub default_layer_bin: Uuid,

	/// The secondary layer bin on the stack that is not the default.
	/// Used for transitioning out of blind mode
	pub transitional_layer_bin: Option<Uuid>,

	/// The order in which to blend layer bins
	pub layer_bin_order: Vec<Uuid>,

	/// The opacity of each layer bin
	pub layer_bin_opacities: HashMap<Uuid, u16>,

	/// The layer bins themselves
	pub layer_bins: HashMap<Uuid, LayerBin>,

	/// The library of submaster values that the mixer pulls from
	pub submasters: HashMap<Uuid, Submaster>,

	/// Output cache for the show. This prevents having to re-blend lights when changes are made
	#[serde(skip)]
	pub output_cache: MixerCache,
}

impl MixerContext {
	pub fn new() -> Self {
		let default_uuid = Uuid::new_v4();

		// Create empty mixer context
		let mut mixer_context = MixerContext {
			default_layer_bin: default_uuid,
			transitional_layer_bin: None,

			layer_bin_order: Vec::new(),
			layer_bin_opacities: HashMap::new(),
			layer_bins: HashMap::new(),

			submasters: HashMap::new(),

			output_cache: Default::default(),
		};

		// Create containers for default layer bin
		mixer_context.layer_bin_order.push(default_uuid);
		mixer_context.layer_bin_opacities.insert(default_uuid, u16::MAX);
		mixer_context.layer_bins.insert(default_uuid, LayerBin {
			layer_order: Vec::new(),
			layer_opacities: HashMap::new(),
		});

		return mixer_context;
	}

	pub fn from_file(mixer_context: MixerContext) -> MixerContext {
		return mixer_context;
	}
}

/// A set of data used to compose groups of layers.
///
/// Useful for things like blind mode, so looks can be created separately and
/// transitioned into & out of
#[portable]
pub struct LayerBin {
	pub layer_order: Vec<Uuid>,
	pub layer_opacities: HashMap<Uuid, u16>,
}

/// Represents the data for all the lights in a layer.
#[portable]
pub struct Submaster {
	pub data: SubmasterData,
}

/// Represents the data within a submaster used for blending
pub type SubmasterData = HashMap<Uuid, AbstractLayerLight>;

/// Represents the abstract data for a single light in a layer.
/// A value's binary may be masked if the output is u8 (integer overflow cast)
pub type AbstractLayerLight = HashMap<String, BlenderValue>;

/// Represents a set of data to be merged with the existing submaster data
pub type SubmasterDelta = HashMap<Uuid, HashMap<String, Option<BlenderValue>>>;

/// Value to be used in a submaster with instructions for mixing it into the result
#[portable]
#[serde(tag = "type", content = "value")]
pub enum BlenderValue {

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

#[derive(Debug, Clone)]
pub struct MixerCache {
	pub layer_bins: HashMap<Uuid, Arc<FullMixerOutput>>,
	pub final_output: Arc<FullMixerOutput>,
}

impl Default for MixerCache {
	fn default() -> Self {
		MixerCache {
			layer_bins: HashMap::new(),
			final_output: Arc::new(HashMap::new()),
		}
	}
}

#[portable]
pub enum BlendingScheme {

	/// Highest Takes Priority (usually used for intensity)
	HTP,

	/// Latest Takes Priority (usually used for color)
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

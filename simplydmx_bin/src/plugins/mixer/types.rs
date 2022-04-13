use std::{
	collections::{
		HashMap,
	},
};

use serde::{
	Serialize,
	Deserialize,
};
use uuid::Uuid;

/// Holds all the information necessary for mixing values for lights
pub type MixerContext {

}

/// Represents the data for all the lights in a layer.
pub type MixerLayer = HashMap<Uuid, AbstractLayerLight>;

/// Represents the abstract data for a single light in a layer.
/// A value's binary may be masked if the output is u8 (integer overflow cast)
pub type AbstractLayerLight = HashMap<String, u16>;

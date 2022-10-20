use std::collections::HashMap;
use simplydmx_plugin_framework::*;
use uuid::Uuid;
use crate::mixer_utils::static_layer::StaticLayer;

// Use this for upgrades: https://serde.rs/attr-default.html

#[portable]
// #[derive(Default)]
pub struct MixerContext {

	/// The default context, where changes are made
	pub default_context: MixingContext,

	/// A frozen copy of the default context used in blind mode
	pub frozen_context: Option<MixingContext>,

	/// The opacity of `default_context` when `frozen_context.is_some()`
	pub blind_opacity: u16,

}

impl Default for MixerContext {
	fn default() -> Self {
		return MixerContext {
			default_context: MixingContext::default(),
			frozen_context: None,
			blind_opacity: 0,
		};
	}
}


impl MixerContext {
	pub fn new() -> Self { MixerContext::default() }

	pub fn from_file(mixer_context: MixerContext) -> MixerContext {
		return mixer_context;
	}
}

#[portable]
pub struct MixingContext {
	pub layer_order: Vec<Uuid>,
	pub layer_opacities: HashMap<Uuid, u16>,
	pub user_submasters: HashMap<Uuid, StaticLayer>,
}
impl Default for MixingContext {
	fn default() -> Self {
		return MixingContext {
			layer_order: Vec::new(),
			layer_opacities: HashMap::new(),
			user_submasters: HashMap::new(),
		};
	}
}

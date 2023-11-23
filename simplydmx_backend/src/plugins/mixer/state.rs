use crate::mixer_utils::{
	layer::MixerLayer,
	state::{FullMixerBlendingData, FullMixerOutput},
	static_layer::StaticLayer,
};
use simplydmx_plugin_framework::*;
use std::collections::HashMap;
use uuid::Uuid;

// Use this for upgrades: https://serde.rs/attr-default.html

#[portable]
/// Data used by the mixer to blend submasters and produce a final result
pub struct MixerContext {
	/// The default context, where changes are made
	pub default_context: MixingContext,

	/// A frozen copy of the default context used in blind mode
	pub frozen_context: Option<MixingContext>,

	/// The opacity of `default_context` when `frozen_context.is_some()`
	pub blind_opacity: u16,
}

impl MixerContext {
	pub async fn cleanup(&mut self, patcher_data: &(FullMixerOutput, FullMixerBlendingData)) {
		for submaster in self.default_context.user_submasters.iter_mut() {
			submaster.1.cleanup(patcher_data).await;
		}
		if let Some(ref mut mixing_context) = self.frozen_context {
			for submaster in mixing_context.user_submasters.iter_mut() {
				submaster.1.cleanup(patcher_data).await;
			}
		}
	}
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
	pub fn new() -> Self {
		MixerContext::default()
	}

	pub fn from_file(mixer_context: MixerContext) -> MixerContext {
		return mixer_context;
	}
}

#[portable]
#[derive(Default)]
/// Defines a set of constraints used for blending a layer
pub struct OpacityGroup {
	pub opacity: u16,
	pub flash_opacity: Option<u16>,
}
impl OpacityGroup {
	pub fn get(&self) -> u16 {
		return self.flash_opacity.unwrap_or(self.opacity);
	}
}

#[portable]
/// Describes a single mixer instance, with its own internal state for driving layers and effects
///
/// Multiple instances are used for creating a blind mode
pub struct MixingContext {
	pub layer_order: Vec<Uuid>,
	pub layer_opacities: HashMap<Uuid, OpacityGroup>,
	pub user_submaster_order: Vec<Uuid>,
	pub user_submasters: HashMap<Uuid, StaticLayer>,
}
impl Default for MixingContext {
	fn default() -> Self {
		return MixingContext {
			layer_order: Vec::new(),
			layer_opacities: HashMap::new(),
			user_submaster_order: Vec::new(),
			user_submasters: HashMap::new(),
		};
	}
}

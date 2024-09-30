use std::sync::Arc;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use shader_context::ShaderContext;

use crate::{
	patcher::fixture_types::{ChannelOffsetValue, ChannelValue},
	utils::{id_alloc::Id, smallmap::SmallMap},
};

pub mod shader_context;

pub static MAX_LAYER_DEPTH: usize = 15;

pub enum UserDefinedValue {
	/// Static value
	Static(ChannelValue),

	/// Offset from previous layer
	Offset(ChannelOffsetValue),

	/// Delegate value calculation to another submaster
	Passthrough(Id),
}

pub enum Submaster {
	UserDefined(SmallMap<(Id, Id), UserDefinedValue>),
	Preset(SmallMap<(Id, Id), UserDefinedValue>),
}
impl Submaster {
	/// This function blends a single channel with an opacity, and emits an output
	pub fn shade(
		&self,
		context: &ShaderContext,
		channel: (Id, Id),
		base: ChannelValue,
		opacity: u16,
		depth: usize,
	) -> ChannelValue {
		if depth > MAX_LAYER_DEPTH {
			return base;
		}
		match self {
			&Submaster::UserDefined(ref values) | &Submaster::Preset(ref values) => values
				.get(&channel)
				.map(|value| match value {
					&UserDefinedValue::Static(new_value) => todo!(),
					&UserDefinedValue::Offset(offset_value) => todo!(),
				})
				.unwrap_or(base),
		}
	}
}

pub struct Layer {
	submaster_id: u32,
	submaster: Arc<Submaster>,
	opacity: u16,
}

pub fn blend_layers(
	base: &SmallMap<(Id, Id), ChannelValue>,
	context: &ShaderContext<'_>,
	layers: &[Layer],
) -> SmallMap<(Id, Id), ChannelValue> {
	base.par_iter()
		.map(|(channel_path, value)| {
			(
				*channel_path,
				layers.iter().fold(*value, |curr, layer| {
					layer
						.submaster
						.shade(context, *channel_path, curr, layer.opacity, 0)
				}),
			)
		})
		.collect()
}

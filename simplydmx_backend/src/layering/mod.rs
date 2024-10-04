use std::cmp;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
	patcher::fixture_types::{
		BlendingScheme, Channel, ChannelOffsetValue, ChannelType, ChannelValue,
	},
	utils::{id_alloc::Id, smallmap::SmallMap},
};

pub static MAX_LAYER_DEPTH: usize = 50;

pub enum UserDefinedValue {
	/// Static value
	Static(ChannelValue),

	/// Offset from previous layer
	Offset(ChannelOffsetValue),

	/// Delegate value calculation to another submaster
	Passthrough(Id),
}

/// Uses `opacity` as a percentage value (0-65535 is 0%-100%) to go from `base` to `overlay`.
/// Used for LTP blending on a per-channel basis
fn blend_ints(base: ChannelValue, overlay: ChannelValue, opacity: u16) -> ChannelValue {
	let base: u16 = base.into();
	let overlay: u16 = overlay.into();
	return ChannelValue::L16(((overlay - base) as u32 * opacity as u32 / 65535u32) as u16 + base);
}

pub fn blend(
	channel_info: &Channel,
	base: ChannelValue,
	value: ChannelValue,
	opacity: u16,
) -> ChannelValue {
	match channel_info.ch_type {
		ChannelType::Segmented {
			segments,
			priority,
			snapping,
		} => {}
		ChannelType::Linear {
			priority: BlendingScheme::HTP,
		} => {}
	}
}

pub enum Submaster {
	UserDefined(SmallMap<(Id, u32), UserDefinedValue>),
	Preset(SmallMap<(Id, u32), UserDefinedValue>),
}
impl Submaster {
	/// This function blends a single channel with an opacity, and emits an output
	pub fn shade(
		&self,
		submasters: &SmallMap<Id, Submaster>,
		channel: (Id, u32),
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
					&UserDefinedValue::Passthrough(submaster_id) => {
						match submasters.get(&submaster_id) {
							Some(submaster) => {
								submaster.shade(submasters, channel, base, opacity, depth + 1)
							}
							None => base,
						}
					}
				})
				.unwrap_or(base),
		}
	}
}

pub struct Layer {
	submaster: u32,
	opacity: u16,
}

pub fn blend_layers(
	base: &SmallMap<(Id, u32), ChannelValue>,
	submasters: &SmallMap<Id, Submaster>,
	layers: &[Layer],
) -> SmallMap<(Id, u32), ChannelValue> {
	base.par_iter()
		.map(|(channel_path, value)| {
			(
				*channel_path,
				layers.iter().fold(*value, |curr, layer| {
					match submasters.get(&layer.submaster) {
						Some(submaster) => {
							submaster.shade(submasters, *channel_path, curr, layer.opacity, 0)
						}
						None => curr,
					}
				}),
			)
		})
		.collect()
}

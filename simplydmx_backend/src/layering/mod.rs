use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[cfg(test)]
mod tests;

use crate::{
	patcher::fixture_types::{
		BlendingScheme, Channel, ChannelOffsetValue, ChannelType, ChannelValue, SnapData,
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

/// Blends a single channel using the default algorithm according to the channel info struct.
pub fn blend(
	channel_info: &Channel,
	base: ChannelValue,
	value: ChannelValue,
	opacity: u16,
) -> ChannelValue {
	match channel_info.ch_type {
		ChannelType::Segmented {
			snapping: SnapData::SnapAt(fulcrum),
			priority: BlendingScheme::LTP,
			..
		} => {
			if opacity > fulcrum.into() {
				value
			} else {
				base
			}
		}
		ChannelType::Segmented {
			snapping: SnapData::SnapAt(fulcrum),
			priority: BlendingScheme::HTP,
			..
		} => {
			if opacity > fulcrum.into() {
				ChannelValue::L16(std::cmp::max(Into::<u16>::into(value), base.into()))
			} else {
				base
			}
		}
		ChannelType::Linear {
			priority: BlendingScheme::HTP,
		}
		| ChannelType::Segmented {
			priority: BlendingScheme::HTP,
			snapping: SnapData::NoSnap,
			..
		} => {
			let value = Into::<u16>::into(value) as u32;
			let opacity = opacity as u32;
			ChannelValue::L16(std::cmp::max(
				base.into(),
				(value * opacity / 65535u32) as u16,
			))
		}
		ChannelType::Linear {
			priority: BlendingScheme::LTP,
		}
		| ChannelType::Segmented {
			priority: BlendingScheme::LTP,
			snapping: SnapData::NoSnap,
			..
		} => {
			let value = Into::<u16>::into(value) as i64;
			let base = Into::<u16>::into(base) as i64;
			ChannelValue::L16(((value - base) * opacity as i64 / 65535i64 + base) as u16)
		}
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

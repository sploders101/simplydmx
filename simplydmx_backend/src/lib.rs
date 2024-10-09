use layering::{Layer, Submaster};
use patcher::{
	fixture_types::{ChannelSize, ChannelValue, FixtureProfile},
	DmxFixtureInstance,
};
use smartstring::{LazyCompact, SmartString};
use utils::{id_alloc::Id, smallmap::SmallMap};

mod layering;
mod patcher;
mod utils;

pub enum DmxDriver {
	/// Dummy output. Universe data will be generated, but will only be used for display
	Dummy,
}

pub struct DmxUniverseInfo {
	pub name: SmartString<LazyCompact>,
	pub driver: DmxDriver,
}

pub struct SimplyDmx {
	/// Library of fixture profiles usable from the patcher
	pub dmx_fixture_library: SmallMap<Id, FixtureProfile>,
	/// Fixtures registered in the show file
	pub dmx_fixtures: SmallMap<Id, DmxFixtureInstance>,
	/// Universes registered in the show file
	pub dmx_universes: SmallMap<Id, DmxUniverseInfo>,
	/// Submasters available for blending via layers
	pub submasters: SmallMap<Id, Submaster>,
	/// Layers used in the final render
	pub layers: SmallMap<Id, Layer>,
}

/// Skips the current iteration of a loop if `$item` is `None`.
///
/// If there's invalid data, better to skip it and move on than to panic.
/// Don't want to plunge everyone into darkness...
/// I hate being the tech guy when that happens...
///
/// This should probably include some logging later.
macro_rules! skip_missing {
	($item:expr) => {
		match $item {
			Some(thing) => thing,
			None => continue,
		}
	};
}

impl SimplyDmx {
	pub fn render_dmx(
		&mut self,
		fixture_data: &SmallMap<(Id, u32), ChannelValue>,
	) -> SmallMap<Id, [u8; 512]> {
		let mut universes = SmallMap::<Id, [u8; 512]>::from_iter(
			self.dmx_universes
				.keys()
				.map(|universe_id| (*universe_id, [0u8; 512])),
		);
		for (fixture_id, fixture_instance) in self.dmx_fixtures.iter() {
			let universe = skip_missing!(universes.get_mut(&fixture_instance.universe));
			let profile = skip_missing!(self
				.dmx_fixture_library
				.get(&fixture_instance.fixture_profile));
			let personality =
				skip_missing!(profile.personalities.get(&fixture_instance.personality));

			// Keep track of how many bytes we've written, since channels can be varying lengths.
			let mut dmx_cursor = fixture_instance.offset;
			let mut channel_inhibitions = SmallMap::<u32, ChannelValue>::default();
			for channel_id in personality.available_channels.iter() {
				let channel_info = skip_missing!(profile.channels.get(channel_id));
				let mut channel_value =
					*skip_missing!(fixture_data.get(&(*fixture_id, *channel_id)));

				// We have everything we need to write the value now, so write it.
				// Don't forget to increment the cursor by the appropriate number of bytes!
				if let Some(ref inhibited_channels) = channel_info.intensity_emulation {
					for channel in inhibited_channels {
						channel_inhibitions.insert(*channel, channel_value);
					}
					continue;
				}

				// Inhibit channel if requested
				if let Some(inhibition) = channel_inhibitions.get(channel_id) {
					let value = Into::<u16>::into(channel_value) as u32;
					let opacity = Into::<u16>::into(*inhibition) as u32;
					channel_value = ChannelValue::L16((value * opacity / 65535u32) as u16);
				}

				match channel_info.size {
					ChannelSize::U8 => {
						// Write one channel
						if dmx_cursor >= 512 {
							// TODO: Add logging. Bad data shouldn't make it this far.
							break;
						}
						universe[dmx_cursor as usize] = (channel_value).into();
						dmx_cursor += 1
					}
					ChannelSize::U16 => {
						// Write two channels
						if dmx_cursor >= 511 {
							// TODO: Add logging. Bad data shouldn't make it this far.
							break;
						}
						let [b1, b2] = Into::<u16>::into(channel_value).to_be_bytes();
						universe[dmx_cursor as usize] = b1;
						universe[dmx_cursor as usize + 1] = b2;
						dmx_cursor += 2
					}
				}
			}
		}
		return universes;
	}
}

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use simplydmx_plugin_framework::*;

use crate::plugins::patcher::driver_plugin_api::{ChannelSize, FixtureInfo, SharablePatcherState};

use super::{
	driver_types::DMXDriver,
	fixture_types::{DMXFixtureData, DMXPersonalityData},
	interface::DMXShowSave,
};

fn get_size(
	personality: &DMXPersonalityData,
	fixture_type_info: &FixtureInfo,
) -> anyhow::Result<u16> {
	// Calculate the fixture & personality's size in the DMX universe
	let mut size = 0;
	for channel_id in personality.dmx_channel_order.iter() {
		let channel = fixture_type_info
			.channels
			.get(channel_id)
			.context("Fixture definition referenced a channel that was not defined.")?;
		size += match channel.size {
			ChannelSize::U8 => 1,
			ChannelSize::U16 => 2,
		};
	}
	return Ok(size);
}

fn check_overlap(range1: (u16, u16), range2: (u16, u16)) -> bool {
	if range1.0 > range2.1 {
		return false;
	} else if range1.1 < range2.0 {
		return false;
	} else {
		return true;
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DMXState {
	#[serde(skip)]
	pub drivers: HashMap<String, Arc<Box<dyn DMXDriver>>>,
	pub library: HashMap<Uuid, DMXFixtureData>,
	pub fixtures: HashMap<Uuid, DMXFixtureInstance>,
	pub universes: HashMap<Uuid, UniverseInstance>,
}
impl DMXState {
	pub fn new() -> Self {
		return DMXState {
			drivers: HashMap::new(),
			library: HashMap::new(),
			fixtures: HashMap::new(),
			universes: HashMap::new(),
		};
	}
	pub fn from_file(file: DMXShowSave) -> Self {
		return DMXState {
			drivers: HashMap::new(),
			library: file.library,
			fixtures: file.fixtures,
			universes: file.universes,
		};
	}
	pub async fn normalize_fixture(
		&self,
		patcher: &SharablePatcherState,
		fixture_type_info: &FixtureInfo,
		personality_id: &str,
		mut fixture: DMXFixtureInstance,
	) -> anyhow::Result<DMXFixtureInstance> {
		match fixture.universe {
			Some(universe) => {
				match fixture.offset {
					Some(offset) => {
						if offset < 1 || offset > 512 {
							return Err(anyhow!("Offset must be between 1 and 512"));
						}

						// Get information about the fixture
						let ctx = self;
						let fixture_info = ctx
							.library
							.get(&fixture_type_info.id)
							.context("Cannot find fixture definition")?;
						let personality = fixture_info
							.personalities
							.get(personality_id)
							.context("Could not find the requested personality")?;

						let size = get_size(personality, fixture_type_info)?;

						// Get the range of values that this fixture would cover (inclusive)
						let covered_range = (offset, offset + size - 1);

						// Check for conflicts
						let overlap = self.fixtures.iter().any(|(fixture_id, fixture)| {
							if fixture.universe != Some(universe) {
								return false;
							}
							if let (Some(offset), Some(patcher_instance)) =
								(fixture.offset, patcher.fixtures.get(fixture_id))
							{
								if let (Some(patcher_definition), Some(dmx_info)) = (
									patcher.library.get(&patcher_instance.fixture_id),
									ctx.library.get(&fixture_type_info.id),
								) {
									if let (Some(dmx_personality),) =
										(dmx_info.personalities.get(&patcher_instance.personality),)
									{
										if let (Ok(size),) =
											(get_size(dmx_personality, patcher_definition),)
										{
											// Now we have offset and size. Check if there's a conflict
											let this_covered_range = (offset, offset + size - 1);
											return check_overlap(
												covered_range,
												this_covered_range,
											);
										}
									}
								}
							}
							return false; // If we had the information, we would have checked already.
						});

						if overlap {
							return Err(anyhow!(
								"Offset conflicts with an existing fixture definition"
							));
						}

						return Ok(fixture);
					}
					None => {
						return Err(anyhow!(
							"Cannot assign fixture to universe without an offset"
						))
					}
				}
			}
			None => {
				fixture.offset = None;
				return Ok(fixture);
			}
		}
	}
}

#[portable]
/// This holds DMX-specific information about a fixture instance
pub struct DMXFixtureInstance {
	pub universe: Option<Uuid>,
	pub offset: Option<u16>,
}

#[portable]
/// This represents a DMX universe instance
pub struct UniverseInstance {
	pub id: Uuid,
	pub name: String,
	pub controller: Option<String>,
}

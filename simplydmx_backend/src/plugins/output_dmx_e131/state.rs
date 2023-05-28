use super::{dmxsource_controller::ControllerCache, interface::E131DMXShowSave};
use crate::plugins::output_dmx::driver_types::DMXFrame;
use simplydmx_plugin_framework::*;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct E131State {
	pub controller: Arc<Mutex<Option<HashMap<u16, DMXFrame>>>>,
	pub universes: HashMap<Uuid, E131Universe>,
}
impl E131State {
	pub fn new(controller_cache: ControllerCache) -> Self {
		return E131State {
			controller: controller_cache,
			universes: HashMap::new(),
		};
	}
	pub fn from_file(controller_cache: ControllerCache, file: E131DMXShowSave) -> Self {
		return E131State {
			controller: controller_cache,
			universes: file.universes,
		};
	}
}

#[portable]
/// E131-specific DMX universe data
pub struct E131Universe {
	pub external_universe: u16,
}

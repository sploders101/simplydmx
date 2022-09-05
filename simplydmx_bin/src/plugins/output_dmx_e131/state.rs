use std::{collections::HashMap, sync::Arc};

use async_std::sync::Mutex;

use simplydmx_plugin_framework::*;
use uuid::Uuid;

use crate::plugins::output_dmx::driver_types::DMXFrame;

use super::dmxsource_controller::initialize_controller;


pub struct E131State {
	pub controller: Arc<Mutex<Option<HashMap<u16, DMXFrame>>>>,
	pub universes: HashMap<Uuid, E131Universe>,
}
impl E131State {
	pub fn new() -> Self {
		return E131State {
			controller: initialize_controller(),
			universes: HashMap::new(),
		}
	}
}

#[portable]
pub struct E131Universe {
	pub external_universe: u16,
}

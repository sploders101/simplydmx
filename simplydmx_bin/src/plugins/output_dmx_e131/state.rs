use std::collections::HashMap;

use async_std::channel::Sender;

use simplydmx_plugin_framework::*;
use uuid::Uuid;

use super::dmxsource_controller::{
	E131Command,
	initialize_controller,
};


pub struct E131State {
	pub controller: Sender<E131Command>,
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

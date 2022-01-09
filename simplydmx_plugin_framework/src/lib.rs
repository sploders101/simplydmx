mod keep_alive;
mod event_emitter;
mod plugin;
mod services;

use std::collections::HashMap;

use self::{
	event_emitter::EventEmitter,
	keep_alive::KeepAlive, plugin::Plugin,
};

pub struct Hypervisor {
	evt_bus: EventEmitter,
	keep_alive: KeepAlive,
	plugins: HashMap<String, Plugin>,
}

pub mod keep_alive;
pub mod event_emitter;
pub mod plugin;
pub mod services;

pub use services::internals::Service;

pub extern crate simplydmx_plugin_macros;
pub use simplydmx_plugin_macros::*;

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

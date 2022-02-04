pub mod keep_alive;
pub mod event_emitter;
pub mod plugin;
pub mod services;

#[cfg(test)]
mod tests;

pub use services::internals::Service;

pub extern crate simplydmx_plugin_macros;
pub use simplydmx_plugin_macros::*;

#[macro_export]
macro_rules! service_docs {
    ($id:literal, $name:literal, $description:literal) => {
        fn get_service_id_internal() -> &'static str {$id}
        fn get_service_name_internal() -> &'static str {$name}
        fn get_service_description_internal() -> &'static str {$description}
    };
}

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

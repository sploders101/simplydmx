use std::{
	collections::HashMap,
	sync::{
		Arc,
	},
	any::Any,
};
use async_std::{
	sync::{
		RwLock,
	},
};
use serde::{
	Serialize,
};
use serde_json::Value;
use crate::{
	event_emitter::{
		EventEmitter,
		EventReceiver,
	},
	keep_alive::KeepAlive,
	services::{
		internals::{
			Service,
			CallServiceError,
			CallServiceJSONError,
		},
		type_specifiers::TypeSpecifier,
	},
};

pub struct Plugin {
	id: String,
	name: String,
	services: RwLock<HashMap<String, Box<dyn Service>>>,
}

pub struct PluginRegistry {
	evt_bus: RwLock<EventEmitter>,
	keep_alive: RwLock<KeepAlive>,
	type_specifiers: RwLock<HashMap<String, Box<dyn TypeSpecifier>>>,
	plugins: RwLock<HashMap<String, Arc<Plugin>>>,
}

pub enum RegisterPluginError {
	IDConflict,
}
pub enum ServiceRegistrationError {
	IDConflict,
}

pub struct PluginContext (Arc<PluginRegistry>, Arc<Plugin>);
impl PluginContext {

	/// Create a new plugin context from a registry arc, a plugin ID, and human-readable plugin name.
	/// This function is only intended to be called by the plugin manager upon instantiation of a plugin
	/// in order to control access to the registry.
	///
	/// Only one plugin can claim an ID at a time. If a plugin tries to register an ID that already exists,
	/// an error will be returned.
	pub async fn new(registry: &Arc<PluginRegistry>, id: String, name: String) -> Result<PluginContext, RegisterPluginError> {
		let mut plugins = registry.plugins.write().await;

		if plugins.contains_key(&id) {
			return Err(RegisterPluginError::IDConflict);
		}

		let plugin = Arc::new(Plugin {
			id: String::clone(&id),
			name: name,
			services: RwLock::new(HashMap::new()),
		});
		plugins.insert(String::clone(&id), Arc::clone(&plugin));

		registry.evt_bus.write().await.send(String::from("simplydmx.plugin_registered"), id).await;

		return Ok(PluginContext (Arc::clone(&registry), plugin));
	}

	/// Register a new service with the system. This service can be discovered and called by other plugins, either by
	/// downcasting to the original type, or using generic call methods implemented by the `Service` trait, allowing
	/// things like user configuration.
	pub async fn register_service<T: Service + 'static>(&self, service: T) -> Result<(), ServiceRegistrationError> {
		let service: Box<dyn Service> = Box::new(service);
		let id = String::from(service.get_id());

		// Register service
		let mut self_services = self.1.services.write().await;
		if self_services.contains_key(&id) {
			return Err(ServiceRegistrationError::IDConflict);
		}
		self_services.insert(String::clone(&id), service);
		drop(self_services);

		// Advertise service via evt_bus
		let mut evt_bus = self.0.evt_bus.write().await;
		evt_bus.send(String::from("simplydmx.service_registered"), String::from(&self.1.id) + "." + &id).await;
		drop(evt_bus);
		return Ok(());
	}

	/// Unregister a service, removing it from any discovery lists
	pub async fn unregister_service(&self, svc_id: &str) {

		// Unregister Service
		let mut self_services = self.1.services.write().await;
		self_services.remove(svc_id);

		// Advertise removal via evt_bus
		let mut evt_bus = self.0.evt_bus.write().await;
		evt_bus.send(String::from("simplydmx.service_removed"), String::from(&self.1.id) + "." + svc_id).await;

	}

	pub async fn call_service(&self, plugin_id: &str, svc_id: &str, args: Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, ExternalServiceError> {

		// Get plugin
		let plugins = self.0.plugins.read().await;
		let plugin = plugins.get(plugin_id);
		if let Some(plugin) = plugin {

			// Get service
			let services = plugin.services.read().await;
			let service = services.get(svc_id);
			if let Some(service) = service {

				// Call service
				let response = service.call(args);
				return match response {
					Ok(value) => Ok(value),
					Err(error) => Err(ExternalServiceError::ErrorReturned(error)),
				};

			} else { return Err(ExternalServiceError::ServiceNotFound) }

		} else { return Err(ExternalServiceError::PluginNotFound) }

	}

	pub async fn call_service_json(&self, plugin_id: &str, svc_id: &str, args: Vec<Value>) -> Result<Value, ExternalServiceJSONError> {

		// Get plugin
		let plugins = self.0.plugins.read().await;
		let plugin = plugins.get(plugin_id);
		if let Some(plugin) = plugin {

			// Get service
			let services = plugin.services.read().await;
			let service = services.get(svc_id);
			if let Some(service) = service {

				// Call service
				let response = service.call_json(args);
				return match response {
					Ok(value) => Ok(value),
					Err(error) => Err(ExternalServiceJSONError::ErrorReturned(error)),
				};

			} else { return Err(ExternalServiceJSONError::ServiceNotFound) }

		} else { return Err(ExternalServiceJSONError::PluginNotFound) }

	}

	/// Sends an event on the bus. `T` gets cast to `Any`, boxed, wrapped in `Arc`,
	/// and sent to all registered listeners.
	pub async fn emit<T: Any + Send + Sync>(&self, event_name: String, message: T) {
		self.0.evt_bus.write().await.send(event_name, message).await;
	}

	/// Registers an event listener on the bus of the given type. Returns
	/// an instance of `EventReceiver<T>` which filters for the desired type
	/// and wraps resulting values in `ArcAny<T>` to make usage of the data
	/// simpler.
	pub async fn on<T: 'static>(&mut self, event_name: String) -> EventReceiver<T> {
		return self.0.evt_bus.write().await.on::<T>(event_name);
	}
}

#[derive(Debug)]
pub enum ExternalServiceError {
	PluginNotFound,
	ServiceNotFound,
	ErrorReturned(CallServiceError),
}

#[derive(Serialize, Debug)]
pub enum ExternalServiceJSONError {
	PluginNotFound,
	ServiceNotFound,
	ErrorReturned(CallServiceJSONError),
}

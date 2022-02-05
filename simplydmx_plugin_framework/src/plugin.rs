use std::{
	collections::HashMap,
	sync::{
		Arc,
	},
};
use async_std::{
	sync::{
		RwLock,
	},
};
use crate::{
	Service,
	event_emitter::EventEmitter,
	keep_alive::KeepAlive,
};

pub struct Plugin {
	id: String,
	name: String,
	services: RwLock<HashMap<String, Box<dyn Service>>>,
}

pub struct PluginRegistry {
	evt_bus: RwLock<EventEmitter>,
	keep_alive: RwLock<KeepAlive>,
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
		plugins.insert(id, Arc::clone(&plugin));

		return Ok(PluginContext (Arc::clone(&registry), plugin));
	}

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

	pub async fn unregister_service(&self, svc_id: &str) {

		// Unregister Service
		let mut self_services = self.1.services.write().await;
		self_services.remove(svc_id);

		// Advertise removal via evt_bus
		let mut evt_bus = self.0.evt_bus.write().await;
		evt_bus.send(String::from("simplydmx.service_removed"), String::from(&self.1.id) + "." + svc_id).await;

	}
}

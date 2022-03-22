use std::{
	collections::{
		HashMap,
		HashSet,
	},
	sync::{
		Arc,
	},
	any::Any,
	future::Future,
};
use uuid::Uuid;
use async_std::{
	sync::{
		RwLock,
	},
	channel::{
		self,
		Sender,
		Receiver,
	},
};
use serde::{
	Serialize,
	Deserialize,
};
use crate::{
	event_emitter::{
		EventEmitter,
		EventReceiver,
	},
	keep_alive::KeepAlive,
	services::{
		internals::{
			Service,
		},
		type_specifiers::{
			TypeSpecifier,
			DropdownOptionNative,
			DropdownOptionJSON,
		},
	},
};

pub use crate::keep_alive::{
	KeepAliveRegistrationError,
	KeepAliveDeregistrationError,
};

pub struct Plugin {
	pub id: String,
	pub name: String,
	services: RwLock<HashMap<String, Arc<Box<dyn Service + Sync + Send>>>>,
	init_flags: RwLock<HashSet<String>>,
}

pub struct PluginRegistry {
	discoverable_services: RwLock<HashMap<String, HashMap<String, ServiceDescription>>>,
	init_bus: RwLock<HashMap<Uuid, Sender<Arc<Dependency>>>>,
	evt_bus: RwLock<EventEmitter>,
	keep_alive: RwLock<KeepAlive>,
	type_specifiers: RwLock<HashMap<String, Box<dyn TypeSpecifier + Sync + Send>>>,
	plugins: RwLock<HashMap<String, Arc<Plugin>>>,
}

/// The plugin manager provides a method of easily instantiating the plugin framework and registering
/// other plugins.
#[derive(Clone)]
pub struct PluginManager(Arc<PluginRegistry>);

impl PluginManager {
	/// Creates a new PluginRegistry, returning a shutdown receiver so the main thread can block,
	/// waiting for a shutdown request, them properly initiate it via the integrated KeepAlive.
	pub fn new() -> (PluginManager, Receiver<()>) {
		let (evt_bus, shutdown_receiver) = EventEmitter::new();
		return (PluginManager(Arc::new(PluginRegistry {
			discoverable_services: RwLock::new(HashMap::new()),
			init_bus: RwLock::new(HashMap::new()),
			evt_bus: RwLock::new(evt_bus),
			keep_alive: RwLock::new(KeepAlive::new()),
			type_specifiers: RwLock::new(HashMap::new()),
			plugins: RwLock::new(HashMap::new()),
		})), shutdown_receiver);
	}

	/// Creates a new plugin context to be passed to a plugin so it can interact with the rest of the
	/// program. The arguments can be anything that can be converted to `String` (like `&'static str`),
	/// for convenience.
	pub async fn register_plugin<S>(&self, id: S, name: S) -> Result<PluginContext, RegisterPluginError>
	where
		S: Into<String>
	{
		return PluginContext::new(&self.0, id.into(), name.into()).await;
	}

	pub async fn shutdown(&self) {
		self.0.evt_bus.write().await.send_shutdown().await;
	}

	pub async fn finish_shutdown(&self) {
		self.0.keep_alive.write().await.shut_down().await;
	}
}

/// This provides a channel through which plugins can communicate with each other and invoke functionality
/// This is the only method through which plugins should be able to communicate with one another. It ensures
/// that all functionality can be used by other plugins like user-made ones, and increases the application's
/// flexibility to communicate through other means (like TCP, for example).
///
/// For example, to expose the ability to shut down the application to other plugins, a "core plugin" could
/// be created with access to both its context, *and* the PluginManager instance, and a service could be
/// created to provide an entrypoint to the shutdown function.
#[derive(Clone)]
pub struct PluginContext (Arc<PluginRegistry>, Arc<Plugin>);
impl PluginContext {


	// ┌──────────────────┐
	// │    Public API    │
	// └──────────────────┘

	/// Create a new plugin context from a registry arc, a plugin ID, and human-readable plugin name.
	/// This function is only intended to be called by the plugin manager upon instantiation of a plugin
	/// in order to control access to the registry.
	///
	/// Only one plugin can claim an ID at a time. If a plugin tries to register an ID that already exists,
	/// an error will be returned.
	async fn new(registry: &Arc<PluginRegistry>, id: String, name: String) -> Result<PluginContext, RegisterPluginError> {

		// Add a slot for discoverable services
		registry.discoverable_services.write().await.insert(String::clone(&id), HashMap::new());

		// Register the plugin
		let mut plugins = registry.plugins.write().await;

		if plugins.contains_key(&id) {
			return Err(RegisterPluginError::IDConflict);
		}

		let plugin = Arc::new(Plugin {
			id: String::clone(&id),
			name: name,
			services: RwLock::new(HashMap::new()),
			init_flags: RwLock::new(HashSet::new()),
		});
		plugins.insert(String::clone(&id), Arc::clone(&plugin));

		// Signal that a new plugin has been registered
		registry.evt_bus.write().await.send(String::from("simplydmx.plugin_registered"), String::clone(&id)).await;

		// Create plugin context
		let plugin_context = PluginContext (Arc::clone(&registry), plugin);
		plugin_context.signal_dep(Dependency::Plugin{ plugin_id: id }).await;

		return Ok(plugin_context);
	}

	/// Set init flag to notify dependents of an initialization step
	pub async fn set_init_flag(&self, flag_name: String) {
		self.1.init_flags.write().await.insert(String::clone(&flag_name));
		let dependency = Dependency::Flag{
			plugin_id: self.1.id.clone(),
			flag_id: flag_name,
		};
		self.signal_dep(dependency).await;
	}

	/// Register a new service with the system. This service can be discovered and called by other plugins, either by
	/// downcasting to the original type, or using generic call methods implemented by the `Service` trait, allowing
	/// things like user configuration.
	pub async fn register_service<T: Service + Sync + Send + 'static>(&self, discoverable: bool, service: T) -> Result<(), ServiceRegistrationError> {
		let service: Box<dyn Service + Sync + Send> = Box::new(service);
		let id = String::from(service.get_id());
		let name = String::from(service.get_name());
		let description = String::from(service.get_description());

		// Register service
		let mut self_services = self.1.services.write().await;
		if self_services.contains_key(&id) {
			return Err(ServiceRegistrationError::IDConflict);
		}
		self_services.insert(String::clone(&id), Arc::new(service));
		drop(self_services);

		// Add service to discoverable list if applicable
		if discoverable {
			self.0.discoverable_services.write().await
				.get_mut(&self.1.id).unwrap() // TODO: Add empty hashmap when plugin is created
				.insert(String::clone(&id), ServiceDescription {
					plugin_id: String::clone(&self.1.id),
					id: String::clone(&id),
					name,
					description,
				});
		}

		// Advertise service via evt_bus
		self.0.evt_bus.write().await
			.send(String::from("simplydmx.service_registered"), String::from(&self.1.id) + "." + &id).await;

		self.signal_dep(Dependency::Service{
			plugin_id: self.1.id.clone(),
			service_id: id,
		}).await;

		return Ok(());
	}

	/// Unregister a service, removing it from any discovery lists
	pub async fn unregister_service(&self, svc_id: &str) {

		// Unregister Service
		self.1.services.write().await.remove(svc_id);

		// Remove service from discoverable list if applicable
		self.0.discoverable_services.write().await
			.get_mut(&self.1.id).unwrap()
			.remove(svc_id);

		// Advertise removal via evt_bus
		self.0.evt_bus.write().await
			.send(String::from("simplydmx.service_removed"), String::from(&self.1.id) + "." + svc_id).await;

	}

	/// List Services
	pub async fn list_services(&self) -> Vec<ServiceDescription> {
		let mut service_list = Vec::new();

		let discoverable_services = self.0.discoverable_services.read().await;
		for plugin in discoverable_services.values() {
			for service in plugin.values() {
				service_list.push(service.clone());
			}
		}

		return service_list;
	}

	/// Get a service object directly
	///
	/// `plugin_id`: The plugin that owns the service
	///
	/// `svc_id`: The ID of the service
	pub async fn get_service(&self, plugin_id: &str, svc_id: &str) -> Result<Arc<Box<dyn Service + Sync + Send>>, GetServiceError> {

		// Get plugin
		let plugins = self.0.plugins.read().await;
		let plugin = plugins.get(plugin_id);
		if let Some(plugin) = plugin {

			// Get service
			let services = plugin.services.read().await;
			let service = services.get(svc_id);
			if let Some(service) = service {

				return Ok(Arc::clone(service));

			} else { return Err(GetServiceError::ServiceNotFound) }

		} else { return Err(GetServiceError::PluginNotFound) }

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

	/// Spawn the specified task when the set of dependencies has finished.
	pub async fn spawn_when<'a, F>(&self, mut dependencies: Vec<Dependency>, blocker: F) -> Result<(), KeepAliveRegistrationError>
	where
		F: Future<Output = ()> + Send + 'static
	{
		// Consolidate list of dependencies to eliminate unnecessary locking
		let mut needed_resources = HashMap::<&String, ConsolidatedDependencies>::new();
		for dependency in dependencies.iter() {
			match dependency {
				Dependency::Flag{ plugin_id, flag_id } => {
					ensure_deplist(&mut needed_resources, plugin_id);
					needed_resources.get_mut(plugin_id).unwrap().flags.push(flag_id);
				},
				Dependency::Plugin{ plugin_id } => {
					ensure_deplist(&mut needed_resources, plugin_id);
					needed_resources.get_mut(plugin_id).unwrap().plugin = true;
				},
				Dependency::Service{ plugin_id, service_id } => {
					ensure_deplist(&mut needed_resources, plugin_id);
					needed_resources.get_mut(plugin_id).unwrap().services.push(service_id);
				},
			}
		}

		// Set up listener channel
		let (sender, receiver) = channel::unbounded::<Arc<Dependency>>();
		let uuid = Uuid::new_v4();
		self.0.init_bus.write().await.insert(uuid.clone(), sender);

		// Create list of fulfilled dependencies
		let mut known_dependencies = Vec::<Dependency>::new();
		let plugins = self.0.plugins.read().await;
		for (plugin_id, deps) in needed_resources {
			let plugin = plugins.get(plugin_id);
			if let Some(plugin) = plugin {
				if deps.plugin {
					known_dependencies.push(Dependency::Plugin{ plugin_id: plugin_id.clone() });
				}
				if deps.services.len() > 0 {
					let services = plugin.services.read().await;
					for service_id in deps.services {
						if services.contains_key(service_id) {
							known_dependencies.push(Dependency::Service{
								plugin_id: plugin_id.clone(),
								service_id: service_id.clone(),
							});
						}
					}
				}
				if deps.flags.len() > 0 {
					let flags = plugin.init_flags.read().await;
					for flag_id in deps.flags {
						if flags.contains(flag_id) {
							known_dependencies.push(Dependency::Flag{
								plugin_id: plugin_id.clone(),
								flag_id: flag_id.clone(),
							});
						}
					}
				}
			}
		}

		// Remove cleared dependencies from original list
		dependencies = dependencies.into_iter().filter(|dependency| known_dependencies.contains(dependency)).collect();

		// Spawn task to finish the process
		return self.spawn(async move {
			// Wait for dependencies to be resolved
			while dependencies.len() > 0 {
				let next_dep = receiver.recv().await.unwrap();
				dependencies = dependencies.into_iter().filter(|dependency| *dependency != *next_dep).collect();
			}

			blocker.await;
		}).await;
	}

	/// Spawns a task that prevents application shutdown until complete.
	///
	/// `blocker`: The future to let finish before shutting down
	pub async fn spawn<F>(&self, blocker: F) -> Result<(), KeepAliveRegistrationError>
	where
		F: Future<Output = ()> + Send + 'static,
	{
		let keep_alive = self.0.keep_alive.write().await;
		return keep_alive.register_blocker(blocker).await;
	}

	/// Registers a future to be driven during the final stage of shutdown. This can be useful for closing
	/// sockets, notifying clients of shutdown, saving files, etc.
	///
	/// `finisher`: The future to drive during shutdown
	pub async fn register_finisher<F>(&self, finisher: F) -> Result<Uuid, KeepAliveRegistrationError>
	where
		F: Future<Output = ()> + Send + 'static,
	{
		let mut keep_alive = self.0.keep_alive.write().await;
		return keep_alive.register_finisher(finisher).await;
	}

	/// Unregisters a previously-registered finisher, removing it from the list of things to do during shutdown
	///
	/// `finisher_id`: The UUID returned from the `register_finisher` call
	pub async fn deregister_finisher<F>(&self, finisher_id: Uuid) -> Result<(), KeepAliveDeregistrationError> {
		let mut keep_alive = self.0.keep_alive.write().await;
		return keep_alive.deregister_finisher(finisher_id).await;
	}

	/// Registers a service type specifier. These can be used to provide dropdown options to graphical interfaces.
	///
	/// `type_id`: The ID of the type specifier, used in a service's `get_signature` function
	///
	/// `type_specifier`: The type specifier to be boxed and stored
	pub async fn register_service_type_specifier<T: TypeSpecifier + Sync + Send + 'static>(&self, type_id: String, type_specifier: T) -> Result<(), TypeSpecifierRegistrationError> {
		let mut type_specifiers = self.0.type_specifiers.write().await;
		if type_specifiers.contains_key(&type_id) {
			type_specifiers.insert(type_id, Box::new(type_specifier));
			return Err(TypeSpecifierRegistrationError::NameConflict);
		} else {
			return Ok(());
		}
	}

	/// Get service type options as native values. This can be used to provide dropdowns for graphical interfaces
	///
	/// `type_id`: The ID of the type specifier, used in a service's `get_signature` function
	///
	/// Returns: `(exclusive, options)`
	///
	/// `exclusive`: Whether or not the user should be allowed to type in their own values
	///
	/// `options`: The options themselves
	pub async fn get_service_type_options(&self, type_id: &str) -> Result<(bool, Vec<DropdownOptionNative>), TypeSpecifierRetrievalError> {
		let type_specifiers = self.0.type_specifiers.read().await;
		if let Some(specifier) = type_specifiers.get(type_id) {
			return Ok((specifier.is_exclusive(), specifier.get_options()));
		} else {
			return Err(TypeSpecifierRetrievalError::SpecifierNotFound);
		}
	}

	/// Get service type options as JSON values. This can be used to provide dropdowns for graphical interfaces
	///
	/// `type_id`: The ID of the type specifier, used in a service's `get_signature` function
	///
	/// Returns: `(exclusive, options)`
	///
	/// `exclusive`: Whether or not the user should be allowed to type in their own values
	///
	/// `options`: The options themselves
	pub async fn get_service_type_options_json(&self, type_id: &str) -> Result<(bool, Vec<DropdownOptionJSON>), TypeSpecifierRetrievalError> {
		let type_specifiers = self.0.type_specifiers.read().await;
		if let Some(specifier) = type_specifiers.get(type_id) {
			return Ok((specifier.is_exclusive(), specifier.get_options_json()));
		} else {
			return Err(TypeSpecifierRetrievalError::SpecifierNotFound);
		}
	}


	// ┌────────────────────────┐
	// │    Helper functions    │
	// └────────────────────────┘

	/// Distribute a dependency to all pending spawn_when tasks
	async fn signal_dep(&self, dep: Dependency) {
		let listeners = self.0.init_bus.read().await;
		let dependency = Arc::new(dep);
		for listener in listeners.values() {
			listener.send(Arc::clone(&dependency)).await.ok();
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TypeSpecifierRegistrationError {
	NameConflict,
}

#[derive(Debug)]
pub enum TypeSpecifierRetrievalError {
	SpecifierNotFound,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceDescription {
	pub plugin_id: String,
	pub id: String,
	pub name: String,
	pub description: String,
}

#[derive(Debug, Serialize)]
pub enum RegisterPluginError {
	IDConflict,
}

#[derive(Debug, Serialize)]
pub enum ServiceRegistrationError {
	IDConflict,
}

#[derive(Debug)]
pub enum GetServiceError {
	PluginNotFound,
	ServiceNotFound,
}

#[derive(PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Dependency {

	/// Represents a dependency on a flag posted by a service, which can represent
	/// any part of the initialization process, like event handlers, for example.
	Flag{ plugin_id: String, flag_id: String },

	/// Represents a service dependency
	Service{ plugin_id: String, service_id: String },

	/// This dependency type is discouraged due to its ambiguous nature.
	/// It only represents a plugin being loaded, and does not garauntee that it
	/// has been properly initialized. It is better to use a `Flag` or `Service`
	/// for this purpose
	Plugin{ plugin_id: String },
}

struct ConsolidatedDependencies<'a> {
	plugin: bool,
	flags: Vec<&'a String>,
	services: Vec<&'a String>,
}

fn ensure_deplist<'a>(deps: &mut HashMap<&'a String, ConsolidatedDependencies>, plugin_id: &'a String) {
	if !deps.contains_key(plugin_id) {
		deps.insert(plugin_id, ConsolidatedDependencies {
			plugin: false,
			flags: Vec::new(),
			services: Vec::new(),
		});
	}
}

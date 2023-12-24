pub mod controller_services;
pub mod providers;
pub mod scalable_value;
pub mod types;
use std::sync::Arc;

use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;
use tokio::sync::RwLock;
use uuid::Uuid;

use self::{
	controller_services::ControllerService,
	providers::{get_providers, ControllerProvider},
	types::Controller,
};

pub struct ControllerInterface(PluginContext, Arc<RwLock<ControllerInterfaceInner>>);
struct ControllerInterfaceInner {
	control_services: FxHashMap<Uuid, Arc<dyn ControllerService + Send + Sync + 'static>>,
	controller_providers: FxHashMap<Uuid, Box<dyn ControllerProvider + Send + Sync + 'static>>,
	controllers: FxHashMap<Uuid, Box<dyn Controller + Send + Sync + 'static>>,
}

impl ControllerInterface {
	pub async fn init(plugin_framework: &PluginManager) -> anyhow::Result<Self> {
		let plugin = plugin_framework
			.register_plugin("live_controller", "Live Controller")
			.await
			.unwrap();

		return Ok(ControllerInterface(
			plugin,
			Arc::new(RwLock::new(ControllerInterfaceInner {
				control_services: Default::default(),
				controller_providers: get_providers(),
				controllers: Default::default(),
			})),
		));
	}

	/// Registers a new controller to be available for linking with control services
	pub async fn register_controller(
		&self,
		uuid: Uuid,
		controller: Box<dyn Controller + Send + Sync + 'static>,
	) -> () {
		let mut ctx = self.1.write().await;
		ctx.controllers.insert(uuid, controller);
	}

	/// Unregisters a controller, unlinking all associated actions
	pub async fn unregister_controller(&self, uuid: &Uuid) {
		let mut ctx = self.1.write().await;
		if let Some(mut old_controller) = ctx.controllers.remove(uuid) {
			old_controller.wait_teardown().await;
		}
	}

	/// Registers a new control service. These should be static and never unloaded. Options can be controlled
	/// using form elements
	pub async fn register_service(
		&self,
		service: Arc<dyn ControllerService + Send + Sync + 'static>,
	) -> Uuid {
		let mut ctx = self.1.write().await;
		let service_id = Uuid::new_v4();
		ctx.control_services.insert(service_id.clone(), service);
		return service_id;
	}
}

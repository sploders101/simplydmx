pub mod types;
pub mod control_interfaces;
pub mod control_proxies;
pub mod scalable_value;
pub mod controller_services;
use std::sync::Arc;

use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;
use tokio::sync::RwLock;
use uuid::Uuid;

use self::{controller_services::ControllerService, types::Controller};

pub struct ControllerInterface(PluginContext, Arc<RwLock<ControllerInterfaceInner>>);
struct ControllerInterfaceInner {
	control_services: FxHashMap<Uuid, Arc<dyn ControllerService + Send + Sync + 'static>>,
	controllers: FxHashMap<Uuid, Controller>,
}

impl ControllerInterface {
	pub async fn init(plugin_framework: &PluginManager) -> anyhow::Result<Self> {
		let plugin = plugin_framework
			.register_plugin("live_controller", "Live Controller")
			.await
			.unwrap();

		return Ok(ControllerInterface(plugin, Arc::new(RwLock::new(ControllerInterfaceInner {
			control_services: Default::default(),
			controllers: Default::default(),
		}))));
	}

	/// Registers a new controller to be available for linking with control services
	pub async fn register_controller(&self, uuid: Uuid, controller: Controller) -> () {
		let mut ctx = self.1.write().await;
		ctx.controllers.insert(uuid, controller);
	}

	/// Unregisters a controller, unlinking all associated actions
	pub async fn unregister_controller(&self, uuid: &Uuid) {
		let mut ctx = self.1.write().await;
		if let Some(old_controller) = ctx.controllers.remove(uuid) {
			for mut control in old_controller.controls.into_values() {
				control.unbind().await;
			}
		}
	}

	/// Registers a new control service. These should be static and never unloaded. Options can be controlled
	/// using form elements
	pub async fn register_service(&self, service: Arc<dyn ControllerService + Send + Sync + 'static>) -> Uuid {
		let mut ctx = self.1.write().await;
		let service_id = Uuid::new_v4();
		ctx.control_services.insert(service_id.clone(), service);
		return service_id;
	}
}

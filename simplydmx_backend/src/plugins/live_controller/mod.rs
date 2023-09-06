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

	// pub async fn register_service
}

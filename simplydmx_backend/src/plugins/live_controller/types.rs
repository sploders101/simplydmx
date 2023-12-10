use simplydmx_plugin_framework::*;

use std::sync::Arc;
use tokio::sync::{Notify, RwLock};
use uuid::Uuid;
use crate::plugins::patcher::driver_plugin_api::AssetDescriptor;
use super::controller_services::ControllerServiceLink;

/// Contains a controller
pub trait Controller {
	fn get_meta<'a>(&'a self) -> &'a ControllerMeta;
	fn get_controls<'a>(&'a self) -> &'a [ControlMeta];
	fn get_control_by_uuid<'a>(&'a self, uuid: &Uuid) -> Option<Arc<Control>>;
}

#[portable]
pub struct ControllerMeta {
	pub name: Arc<str>,
}

#[portable]
/// Metadata that describes a control
pub struct ControlMeta {
	pub id: Uuid,
	pub name: Arc<str>,
	pub description: Option<Arc<str>>,
	pub display: Option<ControlDisplay>,
}

#[portable]
/// Contains information on how to display a control in the UI
pub struct ControlDisplay {
	pub icon: Option<AssetDescriptor>,
	pub x: usize,
	pub y: usize,
	pub width: usize,
	pub height: usize,
}

/// Holds all of a control's state
pub struct Control {
	/// Holds an interface to the service that is currently bound to the control
	binding: RwLock<Option<Box<dyn ControllerServiceLink + Send + Sync + 'static>>>,
	/// Notifies controls they have been re-bound so they can update their receive hooks
	rebind_notifier: Notify,
}
impl Control {
	pub fn new() -> Self {
		return Self {
			binding: RwLock::new(None),
			rebind_notifier: Notify::new(),
		};
	}
	pub async fn unbind(&self) {
		*self.binding.write().await = None;
		self.rebind_notifier.notify_waiters();
	}
}

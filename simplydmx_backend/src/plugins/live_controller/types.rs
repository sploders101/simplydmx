use async_trait::async_trait;
use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;

use super::controller_services::{ControllerLinkDisplay, ControllerServiceLink};
use crate::plugins::patcher::driver_plugin_api::AssetDescriptor;
use std::sync::Arc;
use uuid::Uuid;

/// Contains a controller
#[async_trait]
pub trait Controller {
	fn get_meta<'a>(&'a self) -> &'a ControllerMeta;
	fn get_controls<'a>(&'a self) -> &'a [ControlMeta];
	async fn bind_control(
		&self,
		id: Uuid,
		binding: Option<Box<dyn ControllerServiceLink + Send + Sync + 'static>>,
	);
	async fn get_control_bindings(&self) -> FxHashMap<Uuid, ControllerLinkDisplay>;
	/// Tears down the controller
	///
	/// This cannot be called from box unless it takes a reference, but in concept,
	/// this should take ownership since it invalidates the controller instance.
	async fn wait_teardown(&mut self);
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

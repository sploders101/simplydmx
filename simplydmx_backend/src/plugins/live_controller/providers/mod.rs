mod boards;

use std::sync::Arc;

use async_trait::async_trait;
use rustc_hash::FxHashMap;
use uuid::Uuid;

use crate::{
	plugins::midi_router::MidiRouterInterface,
	utilities::{forms::FormDescriptor, serialized_data::SerializedData},
};

use self::boards::behringer::x_touch::compact::XTouchCompactProvider;

use super::types::{Controller, ControllerMeta};

pub struct ControllerInterfaces {
	midi: MidiRouterInterface,
}

#[async_trait]
pub trait ControllerProvider: Send + Sync + 'static {
	fn id(&self) -> Uuid;
	fn name(&self) -> Arc<str>;
	fn manufacturer(&self) -> Arc<str> {
		Arc::from("Generic")
	}
	fn family(&self) -> Option<Arc<str>> {
		None
	}
	fn display_name(&self) -> Arc<str> {
		Arc::from(format!("{} - {}", self.manufacturer(), self.name()))
	}
	async fn create_form(&self) -> FormDescriptor;
	async fn create_controller(
		&self,
		meta: ControllerMeta,
		form_data: SerializedData,
		interfaces: &ControllerInterfaces,
	) -> anyhow::Result<Box<dyn Controller + Send + Sync + 'static>>;
}

pub fn get_providers() -> FxHashMap<Uuid, Box<dyn ControllerProvider + Send + Sync + 'static>> {
	let mut controllers: FxHashMap<Uuid, Box<dyn ControllerProvider + Send + Sync + 'static>> = FxHashMap::default();

	let x_touch_compact = XTouchCompactProvider;
	controllers.insert(x_touch_compact.id(), Box::new(x_touch_compact));

	return controllers;
}

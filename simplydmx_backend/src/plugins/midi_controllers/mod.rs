mod controller_types;
mod controllers;

use std::sync::Arc;

use anyhow::anyhow;
use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::utilities::{forms::FormDescriptor, serialized_data::SerializedData};

use self::{
	controller_types::{MidiControllerProvider, MidiInterfaceController},
	controllers::behringer::x_touch_compact::XTouchCompact,
};

use super::{
	live_controller::ControllerInterface,
	midi_router::{InputMeta, MidiMomento, MidiRouterInterface, OutputMeta},
};

pub struct MidiControllersInterface(
	PluginContext,
	MidiRouterInterface,
	ControllerInterface,
	Arc<RwLock<MidiControllersInterfaceInner>>,
);
struct MidiControllersInterfaceInner {
	controllers: FxHashMap<Uuid, Arc<dyn MidiControllerProvider + Send + Sync + 'static>>,
	controller_instances: FxHashMap<Uuid, MidiControllerInstanceInfo>,
}

struct MidiControllerInstanceInfo {
	/// The interface controller instance
	interface_controller: Arc<RwLock<MidiInterfaceController>>,
}

#[portable]
pub struct MidiBoardMeta {
	pub name: Arc<str>,
	pub manufacturer: Arc<str>,
	pub family: Option<Arc<str>>,
}

macro_rules! providers {
	(provider $provider:expr) => {
		{
			let provider = $provider;
			(provider.id(), Arc::new(provider) as Arc::<dyn MidiControllerProvider + Send + Sync + 'static>)
		}
	};
	($($provider:expr),+$(,)?) => {
		FxHashMap::from_iter([
			$(providers!(provider $provider)),+
		].into_iter())
	};
}

impl MidiControllersInterface {
	pub async fn init(
		plugin_framework: &PluginManager,
		midi_router: MidiRouterInterface,
		live_control: ControllerInterface,
	) -> Self {
		let plugin = plugin_framework
			.register_plugin("midi-controllers", "Midi Controllers")
			.await
			.unwrap();

		return MidiControllersInterface(
			plugin,
			midi_router,
			live_control,
			Arc::new(RwLock::new(MidiControllersInterfaceInner {
				controllers: providers!(
					XTouchCompact,
				),
				controller_instances: FxHashMap::default(),
			})),
		);
	}

	/// Lists the types of midi boards that the system recognizes
	pub async fn list_boards(&self) -> FxHashMap<Uuid, MidiBoardMeta> {
		return FxHashMap::from_iter(self.3.read().await.controllers.iter().map(
			|(id, provider)| {
				(
					id.clone(),
					MidiBoardMeta {
						name: provider.name(),
						manufacturer: provider.manufacturer(),
						family: provider.family(),
					},
				)
			},
		));
	}

	/// Get the form used for creating a particular board
	pub async fn get_creation_form(&self, board_id: &Uuid) -> Option<FormDescriptor> {
		let ctx = self.3.read().await;
		if let Some(board) = ctx.controllers.get(board_id) {
			return Some(board.create_form().await);
		} else {
			return None;
		}
	}

	/// Create an instance of the requested MIDI board
	pub async fn create_board(
		&self,
		board_id: &Uuid,
		name: Arc<str>,
		form_data: SerializedData,
	) -> anyhow::Result<Uuid> {
		let ctx = self.3.read().await;
		if let Some(board) = ctx.controllers.get(board_id) {
			let interface_controller = MidiInterfaceController::new(
				self.1.clone(),
				InputMeta { name: Arc::clone(&name) },
				MidiMomento::Unlinked,
				Some((OutputMeta { name }, MidiMomento::Unlinked)),
			)
			.await;
			let controller = board.create_controller(form_data, Arc::clone(&interface_controller)).await?;
			let id = Uuid::new_v4();
			self.2.register_controller(id.clone(), controller).await;
			drop(ctx);
			let mut ctx = self.3.write().await;
			ctx.controller_instances.insert(id.clone(), MidiControllerInstanceInfo {
				interface_controller,
			});
			return Ok(id);
		} else {
			return Err(anyhow!("The requested board does not exist."));
		}
	}

	/// Removes a board from the registry, unlinking or deleting all references
	pub async fn delete_board(&self, instance_id: &Uuid) {
		let mut ctx = self.3.write().await;
		self.2.unregister_controller(instance_id).await;
		if let Some(controller_refs) = ctx.controller_instances.remove(instance_id) {
			let mut controller = controller_refs.interface_controller.write().await;
			controller.teardown().await;
		}
	}
}

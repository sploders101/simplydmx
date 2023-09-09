mod controller_types;
mod controllers;

use std::sync::Arc;

use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;
use tokio::sync::RwLock;
use uuid::Uuid;

use self::{
	controller_types::MidiControllerProvider,
	controllers::behringer::x_touch_compact::XTouchCompact,
};

use super::{live_controller::ControllerInterface, midi_router::MidiRouterInterface};

pub struct MidiControllersInterface(
	PluginContext,
	MidiRouterInterface,
	ControllerInterface,
	Arc<RwLock<MidiControllersInterfaceInner>>,
);
struct MidiControllersInterfaceInner {
	controllers: FxHashMap<Uuid, Arc<dyn MidiControllerProvider + Send + Sync + 'static>>,
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
			})),
		);
	}
}

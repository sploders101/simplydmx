mod backends;

use midly::live::LiveEvent;
use rustc_hash::FxHashMap;
pub use simplydmx_plugin_framework::*;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use self::backends::coremidi_backend::CoreMidiBackend;

pub type MidiCallback = Box<dyn FnMut(Vec<u8>) -> () + Send + Sync + 'static>;

pub struct MidiRouterInner {
	internal_sinks: RwLock<FxHashMap<Uuid, Arc<Mutex<MidiCallback>>>>,
	coremidi_backend: CoreMidiBackend,
}

/// The MIDI router is a MIDI interface aggregator. It is responsible for
/// abstracting platform, protocol, and application-specific protocol
/// details. The MIDI router is the central connecting point for all
/// MIDI-based connections such as remote IAC connectors, midir, remote
/// clients, etc.
///
/// This interface also allows quick, user-friendly mapping of virtual
/// interfaces to physical ones.
pub struct MidiRouterInterface {
	plugin: PluginContext,
	inner: Arc<MidiRouterInner>,
}
impl MidiRouterInterface {
	pub async fn init(plugin_framework: &PluginManager) -> anyhow::Result<Self> {
		let plugin = plugin_framework
			.register_plugin("midi_router", "MIDI Router")
			.await
			.unwrap();

		return Ok(MidiRouterInterface {
			plugin,
			inner: Arc::new(MidiRouterInner {
				internal_sinks: RwLock::new(FxHashMap::default()),
				coremidi_backend: CoreMidiBackend::new()?,
			}),
		});
	}
}

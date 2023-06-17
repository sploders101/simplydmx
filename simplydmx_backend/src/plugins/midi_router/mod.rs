mod backends;

use rustc_hash::FxHashMap;
pub use simplydmx_plugin_framework::*;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

#[cfg(feature = "midi-backend-coremidi")]
use self::backends::coremidi_backend::{CoreMidiDestLink, CoreMidiBackend, CoreMidiSourceLink};
use self::backends::{
	SourceLink, DestLink, MidiMomento, AvailableMidiDevice,
};

macro_rules! midi_source {
	($($($feature_name:literal )?$variant_name:ident $inner_type:ty),+$(,)?) => {
		pub enum MidiSource {$(
			$(#[cfg(feature = $feature_name)])?
			$variant_name($inner_type),
		),+}
		impl SourceLink for MidiSource {
			fn get_momento(&self) -> backends::MidiMomento {match *self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(ref connection) => connection.get_momento(),
			),+}}
			fn is_connected(&self) -> bool {match *self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(ref connection) => connection.is_connected(),
			),+}}
			fn unlink(self) {match self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(connection) => connection.unlink(),
			),+}}
		}
	}
}

macro_rules! midi_sink {
	($($($feature_name:literal )?$variant_name:ident $inner_type:ty),+$(,)?) => {
		pub enum MidiSink {$(
			$(#[cfg(feature = $feature_name)])?
			$variant_name($inner_type),
		),+}
		impl DestLink for MidiSink {
			fn get_momento(&self) -> MidiMomento {match *self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(ref connection) => connection.get_momento(),
			),+}}
			fn send_midi(&mut self, data: &[u8]) -> anyhow::Result<()> {match *self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(ref mut connection) => connection.send_midi(data),
			),+}}
			fn disconnect(self) {match self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(connection) => connection.disconnect(),
			),+}}
		}
	}
}

midi_source!(
	"midi-backend-coremidi" CoreMidi CoreMidiSourceLink,
);

midi_sink!(
	"midi-backend-coremidi" CoreMidi CoreMidiDestLink,
);


pub type MidiCallback = Box<dyn FnMut(Vec<u8>) -> () + Send + Sync + 'static>;

pub struct MidiRouterInner {
	internal_sinks: RwLock<FxHashMap<Uuid, Arc<Mutex<MidiCallback>>>>,
	#[cfg(feature = "midi-backend-coremidi")]
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
				#[cfg(feature = "midi-backend-coremidi")]
				coremidi_backend: CoreMidiBackend::new()?,
			}),
		});
	}

	pub async fn list_sources(&self) {
		let mut midi_sources = Vec::<AvailableMidiDevice>::new();

		#[cfg(feature = "midi-backend-coremidi")]
		midi_sources.append(&mut CoreMidiBackend::list_sources());
	}
	pub async fn list_sinks(&self) {
		let mut midi_sinks = Vec::<AvailableMidiDevice>::new();

		#[cfg(feature = "midi-backend-coremidi")]
		midi_sinks.append(&mut CoreMidiBackend::list_sinks());
	}
	// pub async fn create_source
}

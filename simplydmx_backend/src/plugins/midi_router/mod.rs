mod backends;

use async_trait::async_trait;
use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use thiserror::Error;

#[cfg(feature = "midi-backend-coremidi")]
use self::backends::coremidi_backend::{CoreMidiDestLink, CoreMidiBackend, CoreMidiSourceLink};
use self::backends::{SourceLink, DestLink};
pub use self::backends::{
	MidiMomento, AvailableMidiDevice, MidiIndex,
};

/// Creates the `MidiSource` type. This type is supposed to proxy `SourceLink` function
/// calls to any controller while allowing access to the original controller underneath
/// for extra functionality.
macro_rules! midi_source {
	($($($feature_name:literal )?$variant_name:ident $inner_type:ty),+$(,)?) => {
		pub enum MidiSource {$(
			$(#[cfg(feature = $feature_name)])?
			$variant_name($inner_type),
		),+}
		#[async_trait]
		impl SourceLink for MidiSource {
			fn get_momento(&self) -> backends::MidiMomento {match *self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(ref connection) => connection.get_momento(),
			),+}}
			fn is_connected(&self) -> bool {match *self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(ref connection) => connection.is_connected(),
			),+}}
			async fn unlink(self) -> MidiCallback {match self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(connection) => connection.unlink().await,
			),+}}
		}
	}
}

/// Creates the `MidiSink` type. This type is supposed to proxy `DestLink` function
/// calls to any controller while allowing access to the original controller underneath
/// for extra functionality.
macro_rules! midi_sink {
	($($($feature_name:literal )?$variant_name:ident $inner_type:ty),+$(,)?) => {
		pub enum MidiSink {$(
			$(#[cfg(feature = $feature_name)])?
			$variant_name($inner_type),
		),+}
		#[async_trait]
		impl DestLink for MidiSink {
			fn get_momento(&self) -> MidiMomento {match *self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(ref connection) => connection.get_momento(),
			),+}}
			fn send_midi(&mut self, data: &[u8]) -> anyhow::Result<()> {match *self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(ref mut connection) => connection.send_midi(data),
			),+}}
			async fn disconnect(self) {match self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(connection) => connection.disconnect().await,
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

/// Usage: `connect_if!("feature-name", ValueToReturn)`
///
/// Only builds `ValueToReturn` if `"feature-name"` is enabled. Otherwise,
/// immediately returns with `Err(LinkMidiError::DeviceNotFound)`
///
/// Meant for use with functions that take `MidiIndex` to maintain a consistent
/// RPC API
macro_rules! connect_if {
	($feature_name:literal, $func:expr, $disabled_return:expr $(,)?) => {
		{
			#[cfg(feature = $feature_name)]
			{ $func }
			#[cfg(not(feature = $feature_name))]
			{ return $disabled_return; }
		}
	}
}


pub type MidiCallback = Box<dyn Fn(Vec<u8>) -> () + Send + Sync + 'static>;

pub enum InternalSink {
	/// An unlinked sink should contain a MidiCallback to be passed to the
	/// controller when it is eventually linked
	Unlinked(MidiCallback),

	/// A linked sink should contain the MidiSource container that represents
	/// the connection between the sink and the controller's logical device
	Linked(MidiSource),
}

pub enum InternalSource {
	/// An unlinked internal source is simply a placeholder for a MIDI
	/// communication channel, and allows the channel to be linked via a GUI
	/// or API
	Unlinked,
	/// A linked internal source should contain the `MidiSink` container that
	/// represents the connection to the external device
	Linked(MidiSink),
}

pub struct MidiRouterInner {
	internal_sources: RwLock<FxHashMap<Uuid, InternalSource>>,
	internal_sinks: RwLock<FxHashMap<Uuid, InternalSink>>,
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
				internal_sources: RwLock::new(FxHashMap::default()),
				internal_sinks: RwLock::new(FxHashMap::default()),
				#[cfg(feature = "midi-backend-coremidi")]
				coremidi_backend: CoreMidiBackend::new()?,
			}),
		});
	}

	pub async fn list_sources(&self) -> Vec<AvailableMidiDevice> {
		let mut midi_sources = Vec::<AvailableMidiDevice>::new();

		#[cfg(feature = "midi-backend-coremidi")]
		midi_sources.append(&mut CoreMidiBackend::list_sources());

		return midi_sources;
	}
	pub async fn list_sinks(&self) -> Vec<AvailableMidiDevice> {
		let mut midi_sinks = Vec::<AvailableMidiDevice>::new();

		#[cfg(feature = "midi-backend-coremidi")]
		midi_sinks.append(&mut CoreMidiBackend::list_sinks());

		return midi_sinks;
	}
	pub async fn create_source(&self) -> Uuid {
		return self.create_source_with_momento(MidiMomento::Unlinked).await;
	}
	pub async fn create_source_with_momento(&self, momento: MidiMomento) -> Uuid {
		let mut internal_sources = self.inner.internal_sources.write().await;
		let new_uuid = Uuid::new_v4();
		let source = self.get_source_from_momento(momento);
		internal_sources.insert(new_uuid.clone(), source);
		return new_uuid;
	}
	fn get_source_from_momento(&self, momento: MidiMomento) -> InternalSource {
		return match momento {
			MidiMomento::Unlinked => InternalSource::Unlinked,
			MidiMomento::CoreMidi(coremidi_id) => connect_if!(
				"midi-backend-coremidi",
				{
					if let Ok(controller) = self.inner.coremidi_backend.connect_sink(coremidi_id) {
						InternalSource::Linked(MidiSink::CoreMidi(controller))
					} else {
						InternalSource::Unlinked
					}
				},
				InternalSource::Unlinked,
			),
		};
	}
	pub async fn link_source(&self, source_id: &Uuid, device: MidiIndex) -> Result<(), LinkMidiError> {
		let mut internal_sources = self.inner.internal_sources.write().await;
		let source = internal_sources.remove_entry(source_id);

		let uuid = match source {
			None => return Err(LinkMidiError::NotRegistered),
			Some((id, InternalSource::Unlinked)) => id,
			Some((id, InternalSource::Linked(controller))) => {
				controller.disconnect().await;
				id
			}
		};

		let controller = match device {
			MidiIndex::Unlinked => return Ok(()),
			MidiIndex::CoreMidi(midi_id) => connect_if!(
				"midi-backend-coremidi",
				MidiSink::CoreMidi(self.inner.coremidi_backend.connect_sink(midi_id)?),
				Err(LinkMidiError::DeviceNotFound),
			),
		};
		internal_sources.insert(uuid, InternalSource::Linked(controller));

		return Ok(());
	}
	pub async fn remove_source(&self, source_id: &Uuid) {
		let mut internal_sources = self.inner.internal_sources.write().await;
		internal_sources.remove(&source_id);
	}
	pub async fn add_sink(&self, callback: impl Fn(Vec<u8>) -> () + Send + Sync + 'static) -> Uuid {
		return self.add_sink_with_momento(callback, MidiMomento::Unlinked).await;
	}
	pub async fn add_sink_with_momento(&self, callback: impl Fn(Vec<u8>) -> () + Send + Sync + 'static, momento: MidiMomento) -> Uuid {
		let mut internal_sinks = self.inner.internal_sinks.write().await;
		let new_uuid = Uuid::new_v4();
		internal_sinks.insert(new_uuid.clone(), self.get_sink_from_momento(momento, Box::new(callback)));
		return new_uuid;
	}
	pub fn get_sink_from_momento(&self, momento: MidiMomento, callback: MidiCallback) -> InternalSink {
		return match momento {
			MidiMomento::Unlinked => InternalSink::Unlinked(callback),
			MidiMomento::CoreMidi(coremidi_id) => connect_if!(
				"midi-backend-coremidi",
				{
					match self.inner.coremidi_backend.connect_source(coremidi_id, callback) {
						Ok(controller) => {
							InternalSink::Linked(MidiSource::CoreMidi(controller))
						}
						Err((_err, callback)) => {
							InternalSink::Unlinked(callback)
						}
					}
				},
				InternalSink::Unlinked,
			),
		};
	}
	pub async fn link_sink(&self, sink_id: &Uuid, device: MidiIndex) -> Result<(), LinkMidiError> {
		let mut internal_sinks = self.inner.internal_sinks.write().await;
		let sink = internal_sinks.remove_entry(sink_id);

		let (id, callback) = match sink {
			None => return Err(LinkMidiError::NotRegistered),
			Some((id, InternalSink::Unlinked(callback))) => (id, callback),
			Some((id, InternalSink::Linked(controller))) => {
				let callback = controller.unlink().await;
				(id, callback)
			}
		};

		let controller = match device {
			MidiIndex::Unlinked => InternalSink::Unlinked(callback),
			MidiIndex::CoreMidi(coremidi_id) => connect_if!(
				"midi-backend-coremidi",
				{
					match self.inner.coremidi_backend.connect_source(coremidi_id, callback) {
						Ok(controller) => InternalSink::Linked(MidiSource::CoreMidi(controller)),
						Err((err, callback)) => {
							internal_sinks.insert(id, InternalSink::Unlinked(callback));
							return Err(err);
						}
					}
				},
				{
					internal_sinks.insert(id, InternalSink::Unlinked(callback));
					Err(LinkMidiError::DeviceNotFound)
				}
			),
		};
		internal_sinks.insert(id, controller);

		return Ok(());
	}
	pub async fn remove_sink(&self, sink_id: &Uuid) {
		let mut internal_sinks = self.inner.internal_sinks.write().await;
		if let Some(sink) = internal_sinks.remove(sink_id) {
			match sink {
				InternalSink::Unlinked(_) => {}
				InternalSink::Linked(controller) => { let _ = controller.unlink().await; }
			}
		}
	}
}

#[portable]
#[derive(Error)]
pub enum LinkMidiError {
	#[error("The requested logical channel was not registered")]
	NotRegistered,
	#[error("The requested MIDI device doesn't exist.")]
	DeviceNotFound,
	#[error("Unknown error: {0}")]
	Unknown(String),
}

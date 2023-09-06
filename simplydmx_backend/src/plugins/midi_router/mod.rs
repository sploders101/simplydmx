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

/// Creates the `MidiInput` type. This type is supposed to proxy `SourceLink` function
/// calls to any controller while allowing access to the original controller underneath
/// for extra functionality.
macro_rules! midi_source {
	($($($feature_name:literal )?$variant_name:ident $inner_type:ty),+$(,)?) => {
		pub enum MidiInput {$(
			$(#[cfg(feature = $feature_name)])?
			$variant_name($inner_type),
		),+}
		#[async_trait]
		impl SourceLink for MidiInput {
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

/// Creates the `MidiOutput` type. This type is supposed to proxy `DestLink` function
/// calls to any controller while allowing access to the original controller underneath
/// for extra functionality.
macro_rules! midi_sink {
	($($($feature_name:literal )?$variant_name:ident $inner_type:ty),+$(,)?) => {
		pub enum MidiOutput {$(
			$(#[cfg(feature = $feature_name)])?
			$variant_name($inner_type),
		),+}
		#[async_trait]
		impl DestLink for MidiOutput {
			fn get_momento(&self) -> MidiMomento {match *self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(ref connection) => connection.get_momento(),
			),+}}
			fn send_midi(&self, data: &[u8]) -> anyhow::Result<()> {match *self {$(
				$(#[cfg(feature = $feature_name)])?
				Self::$variant_name(ref connection) => connection.send_midi(data),
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

#[portable]
pub struct InputMeta {
	name: String,
}
pub enum LogicalInput {
	/// An unlinked sink should contain a MidiCallback to be passed to the
	/// controller when it is eventually linked
	Unlinked(MidiCallback),

	/// A linked sink should contain the MidiInput container that represents
	/// the connection between the sink and the controller's logical device
	Linked(MidiInput),
}

#[portable]
pub struct OutputMeta {
	name: String,
}
pub enum LogicalOutput {
	/// An unlinked internal source is simply a placeholder for a MIDI
	/// communication channel, and allows the channel to be linked via a GUI
	/// or API
	Unlinked,
	/// A linked internal source should contain the `MidiOutput` container that
	/// represents the connection to the external device
	Linked(MidiOutput),
}

pub struct MidiRouterInner {
	outputs: RwLock<FxHashMap<Uuid, (OutputMeta, LogicalOutput)>>,
	inputs: RwLock<FxHashMap<Uuid, (InputMeta, LogicalInput)>>,
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
#[derive(Clone)]
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
				outputs: RwLock::new(FxHashMap::default()),
				inputs: RwLock::new(FxHashMap::default()),
				#[cfg(feature = "midi-backend-coremidi")]
				coremidi_backend: CoreMidiBackend::new()?,
			}),
		});
	}

	pub async fn list_inputs(&self) -> Vec<AvailableMidiDevice> {
		let mut midi_sources = Vec::<AvailableMidiDevice>::new();

		#[cfg(feature = "midi-backend-coremidi")]
		midi_sources.append(&mut CoreMidiBackend::list_inputs());

		return midi_sources;
	}
	pub async fn list_outputs(&self) -> Vec<AvailableMidiDevice> {
		let mut midi_sinks = Vec::<AvailableMidiDevice>::new();

		#[cfg(feature = "midi-backend-coremidi")]
		midi_sinks.append(&mut CoreMidiBackend::list_outputs());

		return midi_sinks;
	}
	pub async fn create_output(&self, meta: OutputMeta) -> Uuid {
		return self.create_output_with_momento(meta, MidiMomento::Unlinked).await;
	}
	pub async fn create_output_with_momento(&self, meta: OutputMeta, momento: MidiMomento) -> Uuid {
		let mut internal_sources = self.inner.outputs.write().await;
		let new_uuid = Uuid::new_v4();
		let source = self.get_output_from_momento(momento);
		internal_sources.insert(new_uuid.clone(), (meta, source));
		return new_uuid;
	}
	fn get_output_from_momento(&self, momento: MidiMomento) -> LogicalOutput {
		return match momento {
			MidiMomento::Unlinked => LogicalOutput::Unlinked,
			MidiMomento::CoreMidi(coremidi_id) => connect_if!(
				"midi-backend-coremidi",
				{
					if let Ok(controller) = self.inner.coremidi_backend.connect_output(coremidi_id) {
						LogicalOutput::Linked(MidiOutput::CoreMidi(controller))
					} else {
						LogicalOutput::Unlinked
					}
				},
				LogicalOutput::Unlinked,
			),
		};
	}
	pub async fn link_output(&self, output_id: &Uuid, device: MidiIndex) -> Result<(), LinkMidiError> {
		let mut outputs = self.inner.outputs.write().await;
		let output = outputs.remove_entry(output_id);

		let (meta, uuid) = match output {
			None => return Err(LinkMidiError::NotRegistered),
			Some((id, (meta, LogicalOutput::Unlinked))) => (meta, id),
			Some((id, (meta, LogicalOutput::Linked(controller)))) => {
				controller.disconnect().await;
				(meta, id)
			}
		};

		let controller = match device {
			MidiIndex::Unlinked => return Ok(()),
			MidiIndex::CoreMidi(midi_id) => connect_if!(
				"midi-backend-coremidi",
				MidiOutput::CoreMidi(self.inner.coremidi_backend.connect_output(midi_id)?),
				Err(LinkMidiError::DeviceNotFound),
			),
		};
		outputs.insert(uuid, (meta, LogicalOutput::Linked(controller)));

		return Ok(());
	}
	pub async fn send_output(&self, output_id: &Uuid, packet: &[u8]) -> Result<(), SendOutputError> {
		let outputs = self.inner.outputs.read().await;
		match outputs.get(output_id) {
			Some(output) => {
				if let (_, LogicalOutput::Linked(controller)) = output {
					controller.send_midi(packet).map_err(|err| SendOutputError::Unknown(err.to_string()))?;
				}
				return Ok(());
			}
			None => return Err(SendOutputError::NotRegistered),
		}
	}
	pub async fn remove_output(&self, output_id: &Uuid) {
		let mut outputs = self.inner.outputs.write().await;
		outputs.remove(&output_id);
	}
	pub async fn create_input(&self, meta: InputMeta, callback: impl Fn(Vec<u8>) -> () + Send + Sync + 'static) -> Uuid {
		return self.create_input_with_momento(meta, callback, MidiMomento::Unlinked).await;
	}
	pub async fn create_input_with_momento(&self, meta: InputMeta, callback: impl Fn(Vec<u8>) -> () + Send + Sync + 'static, momento: MidiMomento) -> Uuid {
		let mut inputs = self.inner.inputs.write().await;
		let new_uuid = Uuid::new_v4();
		inputs.insert(new_uuid.clone(), (meta, self.get_input_from_momento(momento, Box::new(callback))));
		return new_uuid;
	}
	pub fn get_input_from_momento(&self, momento: MidiMomento, callback: MidiCallback) -> LogicalInput {
		return match momento {
			MidiMomento::Unlinked => LogicalInput::Unlinked(callback),
			MidiMomento::CoreMidi(coremidi_id) => connect_if!(
				"midi-backend-coremidi",
				{
					match self.inner.coremidi_backend.connect_input(coremidi_id, callback) {
						Ok(controller) => {
							LogicalInput::Linked(MidiInput::CoreMidi(controller))
						}
						Err((_err, callback)) => {
							LogicalInput::Unlinked(callback)
						}
					}
				},
				LogicalInput::Unlinked,
			),
		};
	}
	pub async fn link_input(&self, sink_id: &Uuid, device: MidiIndex) -> Result<(), LinkMidiError> {
		let mut inputs = self.inner.inputs.write().await;
		let input = inputs.remove_entry(sink_id);

		let (meta, id, callback) = match input {
			None => return Err(LinkMidiError::NotRegistered),
			Some((id, (meta, LogicalInput::Unlinked(callback)))) => (meta, id, callback),
			Some((id, (meta, LogicalInput::Linked(controller)))) => {
				let callback = controller.unlink().await;
				(meta, id, callback)
			}
		};

		let controller = match device {
			MidiIndex::Unlinked => LogicalInput::Unlinked(callback),
			MidiIndex::CoreMidi(coremidi_id) => connect_if!(
				"midi-backend-coremidi",
				{
					match self.inner.coremidi_backend.connect_input(coremidi_id, callback) {
						Ok(controller) => LogicalInput::Linked(MidiInput::CoreMidi(controller)),
						Err((err, callback)) => {
							inputs.insert(id, (meta, LogicalInput::Unlinked(callback)));
							return Err(err);
						}
					}
				},
				{
					inputs.insert(id, LogicalInput::Unlinked(callback));
					Err(LinkMidiError::DeviceNotFound)
				}
			),
		};
		inputs.insert(id, (meta, controller));

		return Ok(());
	}
	pub async fn remove_input(&self, sink_id: &Uuid) {
		let mut inputs = self.inner.inputs.write().await;
		if let Some(input) = inputs.remove(sink_id) {
			match input {
				(_meta, LogicalInput::Unlinked(_)) => {}
				(_meta, LogicalInput::Linked(controller)) => { let _ = controller.unlink().await; }
			}
		}
	}
}

#[portable]
#[derive(Error)]
/// Represents an error that occurred when linking logical MIDI
/// devices with external ones
pub enum LinkMidiError {
	#[error("The requested logical channel was not registered")]
	NotRegistered,
	#[error("The requested MIDI device doesn't exist.")]
	DeviceNotFound,
	#[error("Unknown error: {0}")]
	Unknown(String),
}

#[portable]
#[derive(Error)]
/// Represents an error that occurred when sending MIDI output
pub enum SendOutputError {
	#[error("The requested logical channel was not registered")]
	NotRegistered,
	#[error("Unknown error: {0}")]
	Unknown(String),
}

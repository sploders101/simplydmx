use simplydmx_plugin_framework::*;

#[cfg(feature = "midi-backend-coremidi")]
pub mod coremidi_backend;

/// Represents a set of criteria or an ID which can be used to search for
/// a midi device upon loading a show file.
///
/// If exactly one device can be found using this criteria, it should be
/// automatically connected.
#[portable]
pub enum MidiMomento {
	CoreMidi(u32),
}

/// Contains information about an available midi device
#[portable]
pub struct AvailableMidiDevice {
	name: String,
	manufacturer: Option<String>,
	id: MidiIndex,
}

/// Represents an ID or index that can be used to retrieve a MIDI device
/// from a backend. This ID should **not** be saved. It does not provide
/// any consistency or validity guarantees across application restarts.
#[portable]
pub enum MidiIndex {
	CoreMidi(u32),
}

pub trait SourceLink {
	/// Returns a "momento", or a chunk of data that can be used when
	/// attempting to locate this device after a restart. Due to MIDI's
	/// nature, a momento is not guaranteed to be unique, and may not always
	/// succeed in locating the device, but should include as much detail
	/// as possible to ensure the highest achievable success rate.
	fn get_momento(&self) -> MidiMomento;
	/// Indicates whether or not the source device this link connects to
	/// is currently attached to the system
	fn is_connected(&self) -> bool;
	/// Unlinks the source from the destination, returning the destination
	/// callback
	fn unlink(self);
}

pub trait DestLink {
	/// Returns a "momento", or a chunk of data that can be used when
	/// attempting to locate this device after a restart. Due to MIDI's
	/// nature, a momento is not guaranteed to be unique, and may not always
	/// succeed in locating the device, but should include as much detail
	/// as possible to ensure the highest achievable success rate.
	fn get_momento(&self) -> MidiMomento;
	/// Sends a midi packet to the destination
	fn send_midi(&mut self, data: &[u8]) -> anyhow::Result<()>;
	/// Disconnects from the destination, consuming self and not allowing
	/// any more packets to be sent
	fn disconnect(self);
}

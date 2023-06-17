use std::sync::Arc;

use anyhow::{anyhow, Context};
use thiserror::Error;
use core_foundation::base::OSStatus;
use coremidi::{Client, InputPortWithContext, OutputPort, EventBuffer, Properties};
use tokio::sync::Mutex;

use crate::plugins::midi_router::MidiCallback;

use super::{SourceLink, AvailableMidiDevice, MidiIndex, DestLink, MidiMomento};

pub struct CoreMidiBackend {
	client: Client,
}
impl CoreMidiBackend {
	pub fn new() -> anyhow::Result<Self> {
		return Ok(Self {
			client: Client::new("SimplyDMX")
				.map_err(|status| CFOSError::from(status))
				.context("An error occured while creating the CoreMIDI backend")?,
		});
	}
	pub fn list_sources() -> Vec<AvailableMidiDevice> {
		return coremidi::Sources.into_iter().filter_map(|source| {
			if let (Some(name), Some(id)) = (source.display_name(), source.unique_id()) {
				return Some(AvailableMidiDevice {
					name,
					manufacturer: source.get_property(&Properties::manufacturer()).ok(),
					id: MidiIndex::CoreMidi(id),
				});
			} else {
				return None;
			}
		}).collect();
	}
	pub fn list_sinks() -> Vec<AvailableMidiDevice> {
		return coremidi::Destinations.into_iter().filter_map(|sink| {
			if let (Some(name), Some(id)) = (sink.display_name(), sink.unique_id()) {
				return Some(AvailableMidiDevice {
					name,
					manufacturer: sink.get_property(&Properties::manufacturer()).ok(),
					id: MidiIndex::CoreMidi(id),
				});
			} else {
				return None;
			}
		}).collect();
	}
	pub fn connect_source(
		&self,
		uid: u32,
		callback: Arc<Mutex<MidiCallback>>,
	) -> anyhow::Result<CoreMidiSourceLink> {
		let source = coremidi::Sources
			.into_iter()
			.find(|source| source.unique_id() == Some(uid));
		if let Some(source) = source {
			let mut input_port = self
				.client
				.input_port_with_protocol(
					&uid.to_string(),
					coremidi::Protocol::Midi10,
					move |event_list, _ctx: &mut ()| {
						let mut cb = callback.blocking_lock();
						for event in event_list.iter() {
							// Convert data into byte slice instead of u32 slice
							let data = event.data();
							// * 4 because u32 is 4 bytes (u8)
							let mut buf = Vec::with_capacity(data.len() * 4);
							for word in data {
								buf.extend(word.to_be_bytes());
							}

							// Push byte slice to callback
							cb(buf);
						}
					},
				)
				.map_err(|status| CFOSError::from(status))
				.context("An error occured while creating the input port")?;
			input_port
				.connect_source(&source, ())
				.map_err(CFOSError::from)
				.context("An error occurred while connecting the input port")?;
			return Ok(CoreMidiSourceLink {
				uid,
				input_port,
				source,
			});
		}
		return Err(anyhow!("Could not find midi source"));
	}
	pub fn connect_sink(
		&self,
		uid: u32,
	) -> anyhow::Result<CoreMidiDestLink> {
		let sink = coremidi::Destinations.into_iter().find(|dest| dest.unique_id() == Some(uid));
		if let Some(sink) = sink {
			let output_port = self.client.output_port(&uid.to_string()).map_err(CFOSError::from).context("An error occurred while creating the output port")?;
			return Ok(CoreMidiDestLink {
				uid,
				output_port,
				sink,
			});
		}
		return Err(anyhow!("Could not find midi destination"));
	}
}

pub struct CoreMidiDestLink {
	uid: u32,
	output_port: OutputPort,
	sink: coremidi::Destination,
}
impl DestLink for CoreMidiDestLink {
	fn get_momento(&self) -> MidiMomento {
		return MidiMomento::CoreMidi(self.uid);
	}
	fn send_midi(&mut self, data: &[u8]) -> anyhow::Result<()> {
		// Everybody else works in bytes (&[u8]), but for some reason, CoreMidi works in &[u32],
		// so we need to re-orient the data to get it ready to send. MIDI is big-endian, so we
		// need to factor this in when building our u32s for CoreMIDI.
		// If statement used until div_ceil is available (https://github.com/rust-lang/rust/issues/88581)
		let mut packet = Vec::<u32>::with_capacity((data.len() / 4) + if data.len() % 4 > 0 { 1 } else { 0 });
		let chunk_iter = data.chunks_exact(4);
		let remainder = chunk_iter.remainder();
		packet.extend(chunk_iter.map(|chunk| {
			let mut new_chunk = [0u8; 4];
			for i in 0..4 {
				new_chunk[i] = chunk[i];
			}
			u32::from_be_bytes(new_chunk)
		}));
		let remainder_len = remainder.len();
		if remainder_len > 0 {
			let mut last_chunk = [0u8; 4];
			for i in 0..(4 - remainder_len) {
				last_chunk[i] = 0;
			}
			for (i, byte) in remainder.iter().enumerate() {
				last_chunk[4 - remainder_len + i] = *byte;
			}
			packet.push(u32::from_be_bytes(last_chunk));
		}

		// Send packet through CoreMIDI
		return Ok(self.output_port
			.send(
				&self.sink,
				EventBuffer::new(coremidi::Protocol::Midi10).with_packet(0, &packet),
			)
			.map_err(|err| CFOSError::from(err))?);
	}

	fn disconnect(self) {
		drop(self);
	}
}


pub struct CoreMidiSourceLink {
	uid: u32,
	input_port: InputPortWithContext<()>,
	source: coremidi::Source,
}
impl SourceLink for CoreMidiSourceLink {
	fn get_momento(&self) -> MidiMomento {
		return MidiMomento::CoreMidi(self.uid);
	}
	fn is_connected(&self) -> bool {
		let result: Result<bool, OSStatus> = self
			.source
			.get_property(&Properties::offline());
		match result {
			Ok(result) => {
				return !result;
			},
			Err(code) => {
				#[cfg(debug_assertions)]
				eprintln!(
					"An error occurred while checking the status of a MIDI device:\n{:?}",
					CFOSError::from(code)
				);
				return false;
			},
		}
	}
	fn unlink(mut self) {
		if let Err(err) = self.input_port.disconnect_source(&self.source) {
			#[cfg(debug_assertions)]
			eprintln!(
				"An error occurred while unlinking a MIDI device:\n{:?}",
				CFOSError::from(err)
			);
		}
	}
}


#[derive(Error, Debug, Clone, Copy)]
pub enum CFOSError {
	#[error("An unknown CoreFoundation error occurred: {0}")]
	Unknown(OSStatus),
}
impl From<OSStatus> for CFOSError {
	fn from(value: OSStatus) -> Self {
		return Self::Unknown(value);
	}
}

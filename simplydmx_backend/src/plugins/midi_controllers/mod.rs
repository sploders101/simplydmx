use simplydmx_plugin_framework::*;

#[portable]
/// Represents a control that communicates via MIDI NoteOn/NoteOff messages
pub struct MidiNote {
	/// channel, note
	pub recv_data: (u8, u8),
	/// channel, note
	pub send_data: Option<(u8, u8)>,
}

#[portable]
/// Represents a control that communicates via MIDI ControlChange messages
pub struct MidiCC {
	/// channel, controlchange
	pub recv_data: (u8, u8),
	/// channel, controlchange
	pub send_data: Option<(u8, u8)>,
}

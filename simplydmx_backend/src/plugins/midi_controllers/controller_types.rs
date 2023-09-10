use std::sync::Arc;

use async_trait::async_trait;
use midly::num::{u4, u7};
use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;
use thiserror::Error;
use tokio::{sync::RwLock, task};
use uuid::Uuid;

use crate::{
	plugins::{
		live_controller::{
			control_interfaces::{Action, AnalogInterface, BooleanInterface},
			scalable_value::ScalableValue,
			types::Controller,
		},
		midi_router::{InputMeta, MidiMomento, MidiRouterInterface, OutputMeta},
	},
	utilities::{forms::FormDescriptor, serialized_data::SerializedData},
};

#[async_trait]
pub trait MidiControllerProvider: Send + Sync + 'static {
	fn id(&self) -> Uuid;
	fn name(&self) -> Arc<str>;
	fn manufacturer(&self) -> Arc<str> { Arc::from("Unknown") }
	fn family(&self) -> Option<Arc<str>> { None }
	async fn create_form(&self) -> FormDescriptor;
	async fn create_controller(
		&self,
		form_data: SerializedData,
		controller: Arc<RwLock<MidiInterfaceController>>,
	) -> anyhow::Result<Controller>;
}

#[portable]
/// Represents a control that communicates via MIDI NoteOn/NoteOff messages
pub struct MidiNote {
	/// channel, note
	pub recv_data: (u8, u8),
	/// channel, note
	pub send_data: Option<(u8, u8)>,
}
impl MidiNote {
	pub fn create(
		&self,
		interface: &mut MidiInterfaceController,
	) -> Result<Arc<MidiNoteControl>, MidiCreationError> {
		if self.recv_data.0 > u4::max_value().as_int() {
			return Err(MidiCreationError::InvalidChannel);
		}
		if self.recv_data.1 > u7::max_value().as_int() {
			return Err(MidiCreationError::InvalidId);
		}
		let note_control = Arc::new(MidiNoteControl {
			midi: interface.get_router(),
			action: RwLock::new(None),
			send_data: if let Some((channel_u8, cc_u8)) = self.send_data {
				if channel_u8 > u4::max_value().as_int() {
					return Err(MidiCreationError::InvalidChannel);
				}
				if cc_u8 > u7::max_value().as_int() {
					return Err(MidiCreationError::InvalidId);
				}
				Some((
					interface
						.get_output_id()
						.ok_or(MidiCreationError::InvalidId)?,
					channel_u8.into(),
					cc_u8.into(),
				))
			} else {
				None
			},
		});
		interface.notes.insert(
			(self.recv_data.0.into(), self.recv_data.1.into()),
			Arc::clone(&note_control),
		);
		return Ok(note_control);
	}
}

#[portable]
/// Represents a control that communicates via MIDI ControlChange messages
pub struct MidiCC {
	/// channel, controlchange
	pub recv_data: (u8, u8),
	/// channel, controlchange
	pub send_data: Option<(u8, u8)>,
}
impl MidiCC {
	pub fn create(
		&self,
		interface: &mut MidiInterfaceController,
	) -> Result<Arc<MidiCCControl>, MidiCreationError> {
		if self.recv_data.0 > u4::max_value().as_int() {
			return Err(MidiCreationError::InvalidChannel);
		}
		if self.recv_data.1 > u7::max_value().as_int() {
			return Err(MidiCreationError::InvalidId);
		}
		let cc_control = Arc::new(MidiCCControl {
			midi: interface.get_router(),
			action: RwLock::new(None),
			send_data: if let Some((channel_u8, cc_u8)) = self.send_data {
				if channel_u8 > u4::max_value().as_int() {
					return Err(MidiCreationError::InvalidChannel);
				}
				if cc_u8 > u7::max_value().as_int() {
					return Err(MidiCreationError::InvalidId);
				}
				Some((
					interface
						.get_output_id()
						.ok_or(MidiCreationError::InvalidId)?,
					channel_u8.into(),
					cc_u8.into(),
				))
			} else {
				None
			},
		});
		interface.controls.insert(
			(self.recv_data.0.into(), self.recv_data.1.into()),
			Arc::clone(&cc_control),
		);
		return Ok(cc_control);
	}
}

#[derive(Debug, Clone, Error)]
pub enum MidiCreationError {
	#[error("Invalid channel")]
	InvalidChannel,
	#[error("Invalid ID")]
	InvalidId,
}

/// A midi interface controller dedicated to routing within the controls framework
pub struct MidiInterfaceController {
	input_id: Option<Uuid>,
	output_id: Option<Uuid>,
	midi: MidiRouterInterface,
	notes: FxHashMap<(u4, u7), Arc<MidiNoteControl>>,
	controls: FxHashMap<(u4, u7), Arc<MidiCCControl>>,
}
impl MidiInterfaceController {
	pub async fn new(
		midi: MidiRouterInterface,
		input_meta: InputMeta,
		input_momento: MidiMomento,
		output_data: Option<(OutputMeta, MidiMomento)>,
	) -> Arc<RwLock<Self>> {
		// Create controller
		let interface = Arc::new(RwLock::new(Self {
			input_id: None,
			output_id: None,
			midi: midi.clone(),
			notes: Default::default(),
			controls: Default::default(),
		}));

		// Create interfaces in the midi router
		let interface_ref = Arc::clone(&interface);
		let input_id = midi
			.create_input_with_momento(
				input_meta,
				move |msg| {
					let interface_ref = Arc::clone(&interface_ref);
					task::spawn(async move {
						interface_ref.read().await.push_msg(&msg).await;
					});
				},
				input_momento,
			)
			.await;
		let mut write_interface = interface.write().await;
		write_interface.input_id = Some(input_id);
		if let Some((output_meta, output_momento)) = output_data {
			let output_id = midi
				.create_output_with_momento(output_meta, output_momento)
				.await;
			write_interface.output_id = Some(output_id);
		}
		drop(write_interface);

		return interface;
	}
	fn get_output_id(&self) -> Option<Uuid> {
		return self.output_id.clone();
	}
	fn get_router(&self) -> MidiRouterInterface {
		return self.midi.clone();
	}
	pub async fn push_msg(&self, msg: &[u8]) {
		match midly::live::LiveEvent::parse(msg) {
			Ok(midly::live::LiveEvent::Midi { channel, message }) => match message {
				midly::MidiMessage::Controller { controller, value } => {
					if let Some(control) = self.controls.get(&(channel, controller)) {
						if let Some(ref action) = *control.action.read().await {
							action(ScalableValue::U7(value));
						}
					}
				}
				midly::MidiMessage::NoteOff { key, vel } => {
					if let Some(control) = self.notes.get(&(channel, key)) {
						if let Some(ref action) = *control.action.read().await {
							action((false, Some(ScalableValue::U7(vel))));
						}
					}
				}
				midly::MidiMessage::NoteOn { key, vel } => {
					if let Some(control) = self.notes.get(&(channel, key)) {
						if let Some(ref action) = *control.action.read().await {
							action((true, Some(ScalableValue::U7(vel))));
						}
					}
				}
				_ => {}
			},
			_ => {}
		}
	}
	pub async fn teardown(&mut self) {
		if let Some(ref input_id) = self.input_id {
			self.midi.remove_input(input_id).await;
		}
		if let Some(ref output_id) = self.output_id {
			self.midi.remove_output(output_id).await;
		}
	}
}

/// A control associated with a midi note
pub struct MidiNoteControl {
	midi: MidiRouterInterface,
	action: RwLock<Option<Action<(bool, Option<ScalableValue>)>>>,
	send_data: Option<(Uuid, u4, u7)>,
}
#[async_trait]
impl BooleanInterface for MidiNoteControl {
	async fn set_bool_action(&self, action: Option<Action<bool>>) {
		*self.action.write().await = match action {
			Some(action) => Some(Box::new(move |(state, _velocity)| action(state))),
			None => None,
		};
	}
	async fn send_bool(&self, state: bool) -> bool {
		self.send_bool_with_velocity((state, ScalableValue::U7(u7::max_value())))
			.await
	}
	async fn set_bool_with_velocity_action(
		&self,
		action: Option<Action<(bool, Option<ScalableValue>)>>,
	) {
		*self.action.write().await = action;
	}
	async fn send_bool_with_velocity(&self, state: (bool, ScalableValue)) -> bool {
		match self.send_data {
			Some((interface_id, channel, key)) => {
				// Send to MidiRouter
				let mut output = Vec::<u8>::new();
				let result = match state.0 {
					true => midly::live::LiveEvent::Midi {
						channel,
						message: midly::MidiMessage::NoteOn {
							key,
							vel: state.1.into(),
						},
					},
					false => midly::live::LiveEvent::Midi {
						channel,
						message: midly::MidiMessage::NoteOff {
							key,
							vel: state.1.into(),
						},
					},
				}
				.write_std(&mut output);
				if let Ok(()) = result {
					if let Ok(()) = self.midi.send_output(&interface_id, &output).await {
						true
					} else {
						false
					}
				} else {
					false
				}
			}
			None => false,
		}
	}
}

/// A control associated with a midi control
pub struct MidiCCControl {
	midi: MidiRouterInterface,
	action: RwLock<Option<Action<ScalableValue>>>,
	send_data: Option<(Uuid, u4, u7)>,
}
#[async_trait]
impl AnalogInterface for MidiCCControl {
	async fn set_analog_action(&self, action: Option<Action<ScalableValue>>) {
		*self.action.write().await = action;
	}
	async fn send_analog(&self, value: ScalableValue) -> bool {
		match self.send_data {
			Some((interface_id, channel, cc)) => {
				let mut output = Vec::<u8>::new();
				let result = midly::live::LiveEvent::Midi {
					channel,
					message: midly::MidiMessage::Controller {
						controller: cc,
						value: value.into(),
					},
				}
				.write_std(&mut output);
				if let Ok(()) = result {
					if let Ok(()) = self.midi.send_output(&interface_id, &output).await {
						true
					} else {
						false
					}
				} else {
					false
				}
			}
			None => false,
		}
	}
}

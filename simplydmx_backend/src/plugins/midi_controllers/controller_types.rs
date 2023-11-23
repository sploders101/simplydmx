use std::sync::Arc;

use async_trait::async_trait;
use rustc_hash::FxHashMap;
use thiserror::Error;
use tokio::{sync::RwLock, task};
use uuid::Uuid;

use crate::{
	plugins::{
		live_controller::{
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
	controls: FxHashMap<Uuid, Arc<Control>>,
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

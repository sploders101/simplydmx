use std::sync::Arc;

use async_trait::async_trait;
use lazy_static::lazy_static;
use midly::{
	live::LiveEvent,
	num::{u4, u7},
	MidiMessage,
};
use rustc_hash::FxHashMap;
use tokio::select;
use uuid::{uuid, Uuid};

use crate::{
	plugins::{
		live_controller::{
			controller_services::{
				ControlUpdate, ControllerButtonEvent, ControllerEvent, ControllerFaderColumnEvent,
				ControllerFaderEvent, ControllerKnobEvent, ControllerLinkDisplay,
				ControllerServiceLink,
			},
			providers::{ControllerInterfaces, ControllerProvider},
			scalable_value::ScalableValue,
			types::{ControlMeta, Controller, ControllerMeta},
		},
		midi_router::{InputMeta, MidiRouterInterface, OutputMeta},
	},
	utilities::{forms::FormDescriptor, serialized_data::SerializedData},
};

macro_rules! controls {
	($($ident:ident $id:literal $name:literal),+$(,)?) => {
		$(static $ident: Uuid = uuid!($id);)+
		lazy_static! {
			static ref CONTROLS: &'static [ControlMeta] = Box::leak(Box::new([
				$(
					ControlMeta {
						id: $ident,
						name: Arc::from($id),
						description: None,
						display: None,
					},
				)+
			]));
		}
	}
}

// Define control metadata
controls! {
	FADER_1        "4663A89C-20F2-4CB8-909E-0BB07232EA02"  "Fader 1",
	FADER_2        "6E2F56B6-7D2E-4A4C-A851-B25C83E0CFCD"  "Fader 2",
	FADER_3        "7071C05B-AAE3-4558-B46D-0FB565DD8F54"  "Fader 3",
	FADER_4        "56F275BD-DAA9-4F8B-8FB1-18BCAC5D9EBA"  "Fader 4",
	FADER_5        "884310F0-2430-4BC1-9BE2-09012B793636"  "Fader 5",
	FADER_6        "A3A53834-EAFA-43D1-B4FE-867CD745B921"  "Fader 6",
	FADER_7        "12DA758E-D237-4E9E-8934-5F12596FC403"  "Fader 7",
	FADER_8        "B70E2E49-821C-4D83-8B4C-5B65AB04E11E"  "Fader 8",
	FADER_9        "5AD73FB0-83DE-487F-AE2E-BA9B6449EC72"  "Fader 9",
	KNOB_1         "F3F153E8-9E32-4D05-A9E3-51C55E70B154"  "Knob 1",
	KNOB_2         "F4E1BBDD-8E30-4572-8D28-9F62E50B122A"  "Knob 2",
	KNOB_3         "A6DC2582-09E7-4E62-A8F9-70241BFB6126"  "Knob 3",
	KNOB_4         "A61DFAFE-4320-4FDF-956F-DF7204785B6F"  "Knob 4",
	KNOB_5         "A7581FC6-E12A-4AD6-A9EC-BAF49CF0A4C5"  "Knob 5",
	KNOB_6         "03A4CCCB-C2CE-4766-A5C3-806DA6608F5B"  "Knob 6",
	KNOB_7         "798C350D-891B-4754-B38A-F404B7C90E11"  "Knob 7",
	KNOB_8         "D010CE07-48F5-4EF1-9697-4680EADA4D97"  "Knob 8",
	KNOB_9         "D8792B3F-C89F-4E4A-B09C-4E428CA20940"  "Knob 9",
	KNOB_10        "E49E3A5A-D913-496F-91C6-C0A85465A5A8"  "Knob 10",
	KNOB_11        "DDCE924B-875F-4045-B454-FD3951357154"  "Knob 11",
	KNOB_12        "20CA1C7A-C6DD-4077-91F6-0B77CC491133"  "Knob 12",
	KNOB_13        "8E9848A9-B1EC-41CA-ABF8-A0F90F72C7A5"  "Knob 13",
	KNOB_14        "9C269774-1F69-4382-8A56-252A79D10AF6"  "Knob 14",
	KNOB_15        "0E458397-9E64-4990-81B0-D71BD2D62570"  "Knob 15",
	KNOB_16        "CF7A3BF8-80C5-4AA6-84C3-568976FDC565"  "Knob 16",
	BUTTON_1       "B7E45052-A453-448F-9746-421209E04E5F"  "Button 1",
	BUTTON_2       "1B9D3EF5-67DA-4153-AE9E-242853FDCC66"  "Button 2",
	BUTTON_3       "886B75B7-7C1B-46B5-9390-BC003BC42D5E"  "Button 3",
	BUTTON_4       "098C11AF-4DE9-4099-97A9-9F9E6D313980"  "Button 4",
	BUTTON_5       "5AC69203-AD9E-4527-9E7C-7877327F9265"  "Button 5",
	BUTTON_6       "C872ADDF-8B0A-4088-9DE4-D37C646ED838"  "Button 6",
	BUTTON_7       "3546D405-D1E4-4CB8-B64F-71DE1B61A360"  "Button 7",
	BUTTON_8       "DC5487F7-34F1-4A14-867D-1BA1BAA3B24B"  "Button 8",
	BUTTON_9       "85156934-1182-45A8-ACCB-B5E36810BA5F"  "Button 9",
	BUTTON_10      "929F6EBC-7630-48CF-9C27-443A7EE4A726"  "Button 10",
	BUTTON_11      "7B8945FC-E390-4704-B062-B0B45EF850A1"  "Button 11",
	BUTTON_12      "3463682F-6475-4F26-A07B-96840C136EFC"  "Button 12",
	BUTTON_13      "E462A2E5-35A1-4FEB-8CCE-84B992B6B2D7"  "Button 13",
	BUTTON_14      "BA7EA164-AC60-47FC-AB95-B86C67C03EFD"  "Button 14",
	BUTTON_15      "F1216BE9-D327-4B42-AFF7-8C6ACE4B841B"  "Button 15",
	BUTTON_16      "DF2DC765-3AA9-40FF-8A87-B35E5E34FD59"  "Button 16",
	BUTTON_17      "53CA128D-3707-4257-BD24-9D79EE8ED7A3"  "Button 17",
	BUTTON_18      "EC26F100-F943-4B35-AA2D-E98F742D5496"  "Button 18",
	BUTTON_19      "DCF03D32-95EE-4340-B1F3-1E338349AD33"  "Button 19",
	BUTTON_20      "6D745720-8682-4EE8-8D66-CC9ECE6DB2F7"  "Button 20",
	BUTTON_21      "8D06DE65-93CA-4506-925B-46607C9B6CC5"  "Button 21",
	BUTTON_22      "6BA3CE8F-E711-4D72-941A-3F4183F99551"  "Button 22",
	BUTTON_23      "3B85FAAC-0D0D-4B13-9D62-83847163943A"  "Button 23",
	BUTTON_24      "C0E5FCCE-A361-4C7D-B061-D5DC44F11BC2"  "Button 24",
	RIGHT_BUTTON_1 "5C34B04D-3D8B-490A-8E46-4E25FE232B76"  "Right Button 1",
	RIGHT_BUTTON_2 "A7B05143-DBBC-4ED9-98A5-F1EFABFE2FA6"  "Right Button 2",
	RIGHT_BUTTON_3 "CF2D5E99-384C-4D81-8D48-33850E9F265B"  "Right Button 3",
	RIGHT_BUTTON_4 "A5A96F66-24C7-426E-96B6-90A935DAEB76"  "Right Button 4",
	RIGHT_BUTTON_5 "F3983309-E728-4797-BD5C-B651B839F1DE"  "Right Button 5",
	RIGHT_BUTTON_6 "41A23307-E1AD-4D2A-8D4D-C4477E997C4D"  "Right Button 6",
}

pub fn create_note_event(state: bool, channel: u4, note: u7) -> Option<(Uuid, ControllerEvent)> {
	macro_rules! control {
		(knob push $control:ident) => {
			Some((
				$control,
				ControllerEvent::Knob(ControllerKnobEvent::Push(state)),
			))
		};
		(button push $control:ident) => {
			Some((
				$control,
				ControllerEvent::Button(ControllerButtonEvent::Push {
					state,
					velocity: None,
				}),
			))
		};
		(fader_column flash $control:ident) => {
			Some((
				$control,
				ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Button(
					ControllerButtonEvent::Push {
						state,
						velocity: None,
					},
				)),
			))
		};
	}
	match (channel.as_int(), note.as_int()) {
		// Knob buttons
		(1, 0) => control!(knob push KNOB_1),
		(1, 1) => control!(knob push KNOB_2),
		(1, 2) => control!(knob push KNOB_3),
		(1, 3) => control!(knob push KNOB_4),
		(1, 4) => control!(knob push KNOB_5),
		(1, 5) => control!(knob push KNOB_6),
		(1, 6) => control!(knob push KNOB_7),
		(1, 7) => control!(knob push KNOB_8),
		(1, 8) => control!(knob push KNOB_9),
		(1, 9) => control!(knob push KNOB_10),
		(1, 10) => control!(knob push KNOB_11),
		(1, 11) => control!(knob push KNOB_12),
		(1, 12) => control!(knob push KNOB_13),
		(1, 13) => control!(knob push KNOB_14),
		(1, 14) => control!(knob push KNOB_15),
		(1, 15) => control!(knob push KNOB_16),

		// 8x3 button field
		(1, 16) => control!(button push BUTTON_1),
		(1, 17) => control!(button push BUTTON_2),
		(1, 18) => control!(button push BUTTON_3),
		(1, 19) => control!(button push BUTTON_4),
		(1, 20) => control!(button push BUTTON_5),
		(1, 21) => control!(button push BUTTON_6),
		(1, 22) => control!(button push BUTTON_7),
		(1, 23) => control!(button push BUTTON_8),
		(1, 24) => control!(button push BUTTON_9),
		(1, 25) => control!(button push BUTTON_10),
		(1, 26) => control!(button push BUTTON_11),
		(1, 27) => control!(button push BUTTON_12),
		(1, 28) => control!(button push BUTTON_13),
		(1, 29) => control!(button push BUTTON_14),
		(1, 30) => control!(button push BUTTON_15),
		(1, 31) => control!(button push BUTTON_16),
		(1, 32) => control!(button push BUTTON_17),
		(1, 33) => control!(button push BUTTON_18),
		(1, 34) => control!(button push BUTTON_19),
		(1, 35) => control!(button push BUTTON_20),
		(1, 36) => control!(button push BUTTON_21),
		(1, 37) => control!(button push BUTTON_22),
		(1, 38) => control!(button push BUTTON_23),
		(1, 39) => control!(button push BUTTON_24),

		// Flash buttons
		(1, 40) => control!(fader_column flash FADER_1),
		(1, 41) => control!(fader_column flash FADER_2),
		(1, 42) => control!(fader_column flash FADER_3),
		(1, 43) => control!(fader_column flash FADER_4),
		(1, 44) => control!(fader_column flash FADER_5),
		(1, 45) => control!(fader_column flash FADER_6),
		(1, 46) => control!(fader_column flash FADER_7),
		(1, 47) => control!(fader_column flash FADER_8),
		(1, 48) => control!(fader_column flash FADER_9),

		// Right buttons
		(1, 49) => control!(button push RIGHT_BUTTON_1),
		(1, 50) => control!(button push RIGHT_BUTTON_2),
		(1, 51) => control!(button push RIGHT_BUTTON_3),
		(1, 52) => control!(button push RIGHT_BUTTON_4),
		(1, 53) => control!(button push RIGHT_BUTTON_5),
		(1, 54) => control!(button push RIGHT_BUTTON_6),
		_ => None,
	}
}

pub fn create_controller_event(
	channel: u4,
	control: u7,
	value: u7,
) -> Option<(Uuid, ControllerEvent)> {
	macro_rules! control {
		(fader move $control:ident) => {
			Some((
				$control,
				ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(
					ControllerFaderEvent::Pos(ScalableValue::U7(value)),
				)),
			))
		};
		(fader touch $control:ident) => {
			Some((
				$control,
				ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(
					ControllerFaderEvent::Touch(value.as_int() > 62),
				)),
			))
		};
		(knob turn $control:ident) => {
			Some((
				$control,
				ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))),
			))
		};
	}
	match (channel.as_int(), control.as_int()) {
		// Faders
		(1, 1) => control!(fader move FADER_1),
		(1, 2) => control!(fader move FADER_2),
		(1, 3) => control!(fader move FADER_3),
		(1, 4) => control!(fader move FADER_4),
		(1, 5) => control!(fader move FADER_5),
		(1, 6) => control!(fader move FADER_6),
		(1, 7) => control!(fader move FADER_7),
		(1, 8) => control!(fader move FADER_8),
		(1, 9) => control!(fader move FADER_9),

		// Fader touches
		(1, 101) => control!(fader touch FADER_1),
		(1, 102) => control!(fader touch FADER_2),
		(1, 103) => control!(fader touch FADER_3),
		(1, 104) => control!(fader touch FADER_4),
		(1, 105) => control!(fader touch FADER_5),
		(1, 106) => control!(fader touch FADER_6),
		(1, 107) => control!(fader touch FADER_7),
		(1, 108) => control!(fader touch FADER_8),
		(1, 109) => control!(fader touch FADER_9),

		// Knobs
		(1, 10) => control!(knob turn KNOB_1),
		(1, 11) => control!(knob turn KNOB_2),
		(1, 12) => control!(knob turn KNOB_3),
		(1, 13) => control!(knob turn KNOB_4),
		(1, 14) => control!(knob turn KNOB_5),
		(1, 15) => control!(knob turn KNOB_6),
		(1, 16) => control!(knob turn KNOB_7),
		(1, 17) => control!(knob turn KNOB_8),
		(1, 18) => control!(knob turn KNOB_9),
		(1, 19) => control!(knob turn KNOB_10),
		(1, 20) => control!(knob turn KNOB_11),
		(1, 21) => control!(knob turn KNOB_12),
		(1, 22) => control!(knob turn KNOB_13),
		(1, 23) => control!(knob turn KNOB_14),
		(1, 24) => control!(knob turn KNOB_15),
		(1, 25) => control!(knob turn KNOB_16),
		_ => None,
	}
}

pub async fn reflect_to_board(
	midi: &MidiRouterInterface,
	midi_out: &Uuid,
	id: &Uuid,
	event: &Arc<dyn ControlUpdate + Send + Sync + 'static>,
) {
	macro_rules! control {
		(fader_column $name:ident fader $fader:literal button $button:literal) => {
			if id == &$name {
				if let Some(status) = event.get_fader() {
					// Set fader position
					let _ = midi.send_output_message(
						midi_out,
						LiveEvent::Midi {
							channel: u4::from_int_lossy(1),
							message: MidiMessage::Controller {
								controller: u7::from_int_lossy($fader),
								value: status.into(),
							},
						},
					)
					.await;
				}
				if let Some(status) = event.get_led() {
					// Set button LED
					let _ = midi.send_output_message(
						midi_out,
						LiveEvent::Midi {
							channel: u4::from_int_lossy(1),
							message: if status {
								MidiMessage::NoteOn {
									key: u7::from_int_lossy($button),
									vel: u7::from_int_lossy(127),
								}
							} else {
								MidiMessage::NoteOff {
									key: u7::from_int_lossy($button),
									vel: u7::from_int_lossy(127),
								}
							},
						},
					)
					.await;
				}
				return;
			}
		};
		(knob $name:ident $knob:literal) => {
			if id == &$name {
				if let Some(status) = event.get_fader() {
					let _ = midi.send_output_message(
						midi_out,
						LiveEvent::Midi {
							channel: u4::from_int_lossy(1),
							message: MidiMessage::Controller {
								controller: u7::from_int_lossy($knob),
								value: status.into(),
							},
						},
					)
					.await;
				}
			}
		};
		(button $name:ident $button:literal) => {
			if id == &$name {
				if let Some(status) = event.get_led() {
					let _ = midi.send_output_message(
						midi_out,
						LiveEvent::Midi {
							channel: u4::from_int_lossy(1),
							message: if status {
								MidiMessage::NoteOn {
									key: u7::from_int_lossy($button),
									vel: u7::from_int_lossy(127),
								}
							} else {
								MidiMessage::NoteOff {
									key: u7::from_int_lossy($button),
									vel: u7::from_int_lossy(127),
								}
							},
						},
					)
					.await;
				}
			}
		};
	}

	// Fader columns
	control!(fader_column FADER_1 fader 1 button 40);
	control!(fader_column FADER_2 fader 2 button 41);
	control!(fader_column FADER_3 fader 3 button 42);
	control!(fader_column FADER_4 fader 4 button 43);
	control!(fader_column FADER_5 fader 5 button 44);
	control!(fader_column FADER_6 fader 6 button 45);
	control!(fader_column FADER_7 fader 7 button 46);
	control!(fader_column FADER_8 fader 8 button 47);
	control!(fader_column FADER_9 fader 9 button 48);

	// Knobs
	control!(knob KNOB_1 0);
	control!(knob KNOB_2 1);
	control!(knob KNOB_3 2);
	control!(knob KNOB_4 3);
	control!(knob KNOB_5 4);
	control!(knob KNOB_6 5);
	control!(knob KNOB_7 6);
	control!(knob KNOB_8 7);
	control!(knob KNOB_9 8);
	control!(knob KNOB_10 9);
	control!(knob KNOB_11 10);
	control!(knob KNOB_12 11);
	control!(knob KNOB_13 12);
	control!(knob KNOB_14 13);
	control!(knob KNOB_15 14);

	// Buttons
	control!(button BUTTON_1 16);
	control!(button BUTTON_2 17);
	control!(button BUTTON_3 18);
	control!(button BUTTON_4 19);
	control!(button BUTTON_5 20);
	control!(button BUTTON_6 21);
	control!(button BUTTON_7 22);
	control!(button BUTTON_8 23);
	control!(button BUTTON_9 24);
	control!(button BUTTON_10 25);
	control!(button BUTTON_11 26);
	control!(button BUTTON_12 27);
	control!(button BUTTON_13 28);
	control!(button BUTTON_14 29);
	control!(button BUTTON_15 30);
	control!(button BUTTON_16 31);
	control!(button BUTTON_17 32);
	control!(button BUTTON_18 33);
	control!(button BUTTON_19 34);
	control!(button BUTTON_20 35);
	control!(button BUTTON_21 36);
	control!(button BUTTON_22 37);
	control!(button BUTTON_23 38);
	control!(button BUTTON_24 39);

	control!(button RIGHT_BUTTON_1 49);
	control!(button RIGHT_BUTTON_2 50);
	control!(button RIGHT_BUTTON_3 51);
	control!(button RIGHT_BUTTON_4 52);
	control!(button RIGHT_BUTTON_5 53);
	control!(button RIGHT_BUTTON_6 54);
}

pub struct XTouchCompactProvider;
#[async_trait]
impl ControllerProvider for XTouchCompactProvider {
	fn id(&self) -> Uuid {
		uuid!("569EC2D6-E5B4-4FB8-B27D-C17F41728F6B")
	}
	fn name(&self) -> Arc<str> {
		Arc::from("X-Touch Compact")
	}
	fn manufacturer(&self) -> Arc<str> {
		Arc::from("Behringer")
	}
	fn family(&self) -> Option<Arc<str>> {
		Some(Arc::from("X-Touch"))
	}
	async fn create_form(&self) -> FormDescriptor {
		return FormDescriptor::new();
	}

	async fn create_controller(
		&self,
		meta: ControllerMeta,
		_form_data: SerializedData,
		interfaces: &ControllerInterfaces,
	) -> anyhow::Result<Box<dyn Controller + Send + Sync + 'static>> {
		return Ok(Box::new(
			XTouchCompact::new(meta, interfaces.midi.clone()).await?,
		));
	}
}

pub struct XTouchCompact {
	// id: Uuid,
	meta: ControllerMeta,
	// midi: MidiRouterInterface,
	// midi_in: Uuid,
	// midi_out: Uuid,
	bind_channel: tokio::sync::mpsc::Sender<(
		Uuid,
		Option<Box<dyn ControllerServiceLink + Send + Sync + 'static>>,
		tokio::sync::oneshot::Sender<()>,
	)>,
	link_display_channel: tokio::sync::mpsc::Sender<
		tokio::sync::oneshot::Sender<FxHashMap<Uuid, ControllerLinkDisplay>>,
	>,
	controller_task: Option<tokio::task::JoinHandle<()>>,
	shutdown_sender: tokio::sync::mpsc::Sender<()>,
}
impl XTouchCompact {
	pub async fn new(meta: ControllerMeta, midi: MidiRouterInterface) -> anyhow::Result<Self> {
		let controller_id = Uuid::new_v4();
		let (event_sender, mut event_receiver) = tokio::sync::mpsc::unbounded_channel();
		let (bind_channel, mut bind_receiver) = tokio::sync::mpsc::channel::<(
			Uuid,
			Option<Box<dyn ControllerServiceLink + Send + Sync + 'static>>,
			tokio::sync::oneshot::Sender<()>,
		)>(1);
		let (link_display_channel, mut link_display_receiver) = tokio::sync::mpsc::channel::<
			tokio::sync::oneshot::Sender<FxHashMap<Uuid, ControllerLinkDisplay>>,
		>(1);

		let midi_in = midi
			.create_input(
				InputMeta {
					name: Arc::clone(&meta.name),
					group: Some(controller_id.clone()),
				},
				move |packet| {
					if let Some((control, event)) = match LiveEvent::parse(&packet) {
						Ok(LiveEvent::Midi { channel, message }) => match message {
							MidiMessage::NoteOff { key, .. } => {
								create_note_event(false, channel, key)
							}
							MidiMessage::NoteOn { key, .. } => {
								create_note_event(true, channel, key)
							}
							MidiMessage::Controller { controller, value } => {
								create_controller_event(channel, controller, value)
							}
							_ => None,
						},
						_ => None,
					} {
						// Emit control, event
						let _ = event_sender.send((control, event));
					}
				},
			)
			.await;

		let midi_out = midi
			.create_output(OutputMeta {
				name: Arc::clone(&meta.name),
				group: Some(controller_id.clone()),
			})
			.await;

		// Spawn controller task
		let (shutdown_sender, mut shutdown_receiver) = tokio::sync::mpsc::channel(1);
		let controller_task = {
			let midi = midi.clone();
			let midi_in = midi_in.clone();
			let midi_out = midi_out.clone();
			tokio::task::spawn(async move {
				let mut bindings = FxHashMap::<
					Uuid,
					Box<dyn ControllerServiceLink + Send + Sync + 'static>,
				>::default();

				// TODO: Change this to a `watch` channel with merging `ControlUpdate` interfaces
				// This channel is used to aggregate all control updates in one place that's easy
				// to manage
				let (reflector_sender, mut reflector_receiver) = tokio::sync::mpsc::unbounded_channel::<(
					Uuid,
					Arc<dyn ControlUpdate + Send + Sync + 'static>,
				)>();

				loop {
					select! {
						biased; // Actions are in order of priority

						// I/O
						Some((control, event)) = event_receiver.recv() => if let Some(binding) = bindings.get(&control) {
							binding.emit(event).await;
						},
						Some((control, update)) = reflector_receiver.recv() => reflect_to_board(&midi, &midi_out, &control, &update).await,

						// Binding updates
						Some((control, binding, callback)) = bind_receiver.recv() => {
							match binding {
								Some(binding) => {
									// Set up the update channel
									let reflector_sender = reflector_sender.clone();
									binding.set_update_channel(Box::new(move |update| {
										let _ = reflector_sender.send((control.clone(), update));
									})).await;

									// Replace old binding, unlinking it if there was one
									if let Some(old_binding) = bindings.insert(control, binding) {
										old_binding.unlink().await;
									}

									// Signal to the caller that we finished updating the binding
									let _ = callback.send(());
								}
								None => {
									if let Some(old_binding) = bindings.remove(&control) {
										old_binding.unlink().await;
									}
								}
							}
						}

						// Handle requests for UI data
						Some(callback) = link_display_receiver.recv() => {
							let _ = callback.send(
								FxHashMap::from_iter(
									bindings
										.iter()
										.map(|(id, binding)| (id.clone(), binding.display_data()))
								)
							);
						}

						event = shutdown_receiver.recv() => {
							if event.is_none() {
								#[cfg(debug_assertions)]
								eprintln!("Controller dropped without shutdown");
							}
							break;
						}
					}
				}

				// Cleanup
				let mut cleanup_tasks = tokio::task::JoinSet::<()>::new();
				for link in bindings.into_values() {
					cleanup_tasks.spawn(async move {
						link.unlink().await;
					});
				}
				tokio::join!(
					midi.remove_output(&midi_out),
					midi.remove_input(&midi_in),
					async move { while let Some(_) = cleanup_tasks.join_next().await {} },
				);
			})
		};

		return Ok(Self {
			// id: controller_id,
			meta,
			// midi_in,
			// midi_out,
			// midi,
			bind_channel,
			link_display_channel,
			controller_task: Some(controller_task),
			shutdown_sender,
		});
	}
}
#[async_trait]
impl Controller for XTouchCompact {
	fn get_meta<'a>(&'a self) -> &'a ControllerMeta {
		return &self.meta;
	}
	fn get_controls<'a>(&'a self) -> &'a [ControlMeta] {
		return &CONTROLS;
	}
	async fn bind_control(
		&self,
		id: Uuid,
		binding: Option<Box<dyn ControllerServiceLink + Send + Sync + 'static>>,
	) {
		let (sender, receiver) = tokio::sync::oneshot::channel();
		let _ = self.bind_channel.send((id, binding, sender)).await;
		let _ = receiver.await;
	}
	async fn get_control_bindings(&self) -> FxHashMap<Uuid, ControllerLinkDisplay> {
		let (sender, receiver) = tokio::sync::oneshot::channel();
		self.link_display_channel.send(sender).await.unwrap();
		return receiver.await.unwrap();
	}
	async fn wait_teardown(&mut self) {
		let _ = self.shutdown_sender.send(()).await;
		if let Some(task) = self.controller_task.take() {
			let _ = task.await;
		} else {
			#[cfg(debug_assertions)]
			eprintln!("Controller teardown called twice");
		}
	}
}

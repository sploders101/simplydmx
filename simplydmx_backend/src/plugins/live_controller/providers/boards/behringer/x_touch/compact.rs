use std::sync::Arc;

use async_trait::async_trait;
use midly::{num::{u4, u7}, live::LiveEvent, MidiMessage};
use rustc_hash::FxHashMap;
use uuid::{uuid, Uuid};

use crate::{
	plugins::{live_controller::{
		scalable_value::ScalableValue,
		types::{Control, Controller, ControllerMeta, ControlMeta}, providers::{ControllerProvider, ControllerInterfaces}, controller_services::{ControllerEvent, ControllerButtonEvent, ControllerKnobEvent, ControllerFaderColumnEvent, ControllerFaderEvent},
	}, midi_router::{InputMeta, MidiRouterInterface, OutputMeta}},
	utilities::{
		forms::FormDescriptor,
		serialized_data::SerializedData,
	},
};

static FADER_1: Uuid = uuid!("4663A89C-20F2-4CB8-909E-0BB07232EA02");
static FADER_2: Uuid = uuid!("6E2F56B6-7D2E-4A4C-A851-B25C83E0CFCD");
static FADER_3: Uuid = uuid!("7071C05B-AAE3-4558-B46D-0FB565DD8F54");
static FADER_4: Uuid = uuid!("56F275BD-DAA9-4F8B-8FB1-18BCAC5D9EBA");
static FADER_5: Uuid = uuid!("884310F0-2430-4BC1-9BE2-09012B793636");
static FADER_6: Uuid = uuid!("A3A53834-EAFA-43D1-B4FE-867CD745B921");
static FADER_7: Uuid = uuid!("12DA758E-D237-4E9E-8934-5F12596FC403");
static FADER_8: Uuid = uuid!("B70E2E49-821C-4D83-8B4C-5B65AB04E11E");
static FADER_9: Uuid = uuid!("5AD73FB0-83DE-487F-AE2E-BA9B6449EC72");
static KNOB_1: Uuid = uuid!("F3F153E8-9E32-4D05-A9E3-51C55E70B154");
static KNOB_2: Uuid = uuid!("F4E1BBDD-8E30-4572-8D28-9F62E50B122A");
static KNOB_3: Uuid = uuid!("A6DC2582-09E7-4E62-A8F9-70241BFB6126");
static KNOB_4: Uuid = uuid!("A61DFAFE-4320-4FDF-956F-DF7204785B6F");
static KNOB_5: Uuid = uuid!("A7581FC6-E12A-4AD6-A9EC-BAF49CF0A4C5");
static KNOB_6: Uuid = uuid!("03A4CCCB-C2CE-4766-A5C3-806DA6608F5B");
static KNOB_7: Uuid = uuid!("798C350D-891B-4754-B38A-F404B7C90E11");
static KNOB_8: Uuid = uuid!("D010CE07-48F5-4EF1-9697-4680EADA4D97");
static KNOB_9: Uuid = uuid!("D8792B3F-C89F-4E4A-B09C-4E428CA20940");
static KNOB_10: Uuid = uuid!("E49E3A5A-D913-496F-91C6-C0A85465A5A8");
static KNOB_11: Uuid = uuid!("DDCE924B-875F-4045-B454-FD3951357154");
static KNOB_12: Uuid = uuid!("20CA1C7A-C6DD-4077-91F6-0B77CC491133");
static KNOB_13: Uuid = uuid!("8E9848A9-B1EC-41CA-ABF8-A0F90F72C7A5");
static KNOB_14: Uuid = uuid!("9C269774-1F69-4382-8A56-252A79D10AF6");
static KNOB_15: Uuid = uuid!("0E458397-9E64-4990-81B0-D71BD2D62570");
static KNOB_16: Uuid = uuid!("CF7A3BF8-80C5-4AA6-84C3-568976FDC565");
static BUTTON_1: Uuid = uuid!("B7E45052-A453-448F-9746-421209E04E5F");
static BUTTON_2: Uuid = uuid!("1B9D3EF5-67DA-4153-AE9E-242853FDCC66");
static BUTTON_3: Uuid = uuid!("886B75B7-7C1B-46B5-9390-BC003BC42D5E");
static BUTTON_4: Uuid = uuid!("098C11AF-4DE9-4099-97A9-9F9E6D313980");
static BUTTON_5: Uuid = uuid!("5AC69203-AD9E-4527-9E7C-7877327F9265");
static BUTTON_6: Uuid = uuid!("C872ADDF-8B0A-4088-9DE4-D37C646ED838");
static BUTTON_7: Uuid = uuid!("3546D405-D1E4-4CB8-B64F-71DE1B61A360");
static BUTTON_8: Uuid = uuid!("DC5487F7-34F1-4A14-867D-1BA1BAA3B24B");
static BUTTON_9: Uuid = uuid!("85156934-1182-45A8-ACCB-B5E36810BA5F");
static BUTTON_10: Uuid = uuid!("929F6EBC-7630-48CF-9C27-443A7EE4A726");
static BUTTON_11: Uuid = uuid!("7B8945FC-E390-4704-B062-B0B45EF850A1");
static BUTTON_12: Uuid = uuid!("3463682F-6475-4F26-A07B-96840C136EFC");
static BUTTON_13: Uuid = uuid!("E462A2E5-35A1-4FEB-8CCE-84B992B6B2D7");
static BUTTON_14: Uuid = uuid!("BA7EA164-AC60-47FC-AB95-B86C67C03EFD");
static BUTTON_15: Uuid = uuid!("F1216BE9-D327-4B42-AFF7-8C6ACE4B841B");
static BUTTON_16: Uuid = uuid!("DF2DC765-3AA9-40FF-8A87-B35E5E34FD59");
static BUTTON_17: Uuid = uuid!("53CA128D-3707-4257-BD24-9D79EE8ED7A3");
static BUTTON_18: Uuid = uuid!("EC26F100-F943-4B35-AA2D-E98F742D5496");
static BUTTON_19: Uuid = uuid!("DCF03D32-95EE-4340-B1F3-1E338349AD33");
static BUTTON_20: Uuid = uuid!("6D745720-8682-4EE8-8D66-CC9ECE6DB2F7");
static BUTTON_21: Uuid = uuid!("8D06DE65-93CA-4506-925B-46607C9B6CC5");
static BUTTON_22: Uuid = uuid!("6BA3CE8F-E711-4D72-941A-3F4183F99551");
static BUTTON_23: Uuid = uuid!("3B85FAAC-0D0D-4B13-9D62-83847163943A");
static BUTTON_24: Uuid = uuid!("C0E5FCCE-A361-4C7D-B061-D5DC44F11BC2");
static RIGHT_BUTTON_1: Uuid = uuid!("5C34B04D-3D8B-490A-8E46-4E25FE232B76");
static RIGHT_BUTTON_2: Uuid = uuid!("A7B05143-DBBC-4ED9-98A5-F1EFABFE2FA6");
static RIGHT_BUTTON_3: Uuid = uuid!("CF2D5E99-384C-4D81-8D48-33850E9F265B");
static RIGHT_BUTTON_4: Uuid = uuid!("A5A96F66-24C7-426E-96B6-90A935DAEB76");
static RIGHT_BUTTON_5: Uuid = uuid!("F3983309-E728-4797-BD5C-B651B839F1DE");
static RIGHT_BUTTON_6: Uuid = uuid!("41A23307-E1AD-4D2A-8D4D-C4477E997C4D");
static CONTROLS: [ControlMeta; 0] = [];


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
		return Ok(Box::new(XTouchCompact::new(
			meta,
			interfaces.midi.clone(),
		).await?));
	}
}

pub fn create_note_event(state: bool, channel: u4, note: u7) -> Option<(Uuid, ControllerEvent)> {
	match (channel.as_int(), note.as_int()) {
		// Knob buttons
		(1, 0) => Some((KNOB_1, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 1) => Some((KNOB_2, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 2) => Some((KNOB_3, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 3) => Some((KNOB_4, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 4) => Some((KNOB_5, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 5) => Some((KNOB_6, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 6) => Some((KNOB_7, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 7) => Some((KNOB_8, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 8) => Some((KNOB_9, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 9) => Some((KNOB_10, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 10) => Some((KNOB_11, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 11) => Some((KNOB_12, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 12) => Some((KNOB_13, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 13) => Some((KNOB_14, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 14) => Some((KNOB_15, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),
		(1, 15) => Some((KNOB_16, ControllerEvent::Knob(ControllerKnobEvent::Push(state)))),

		// 8x3 button field
		(1, 16) => Some((BUTTON_1, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 17) => Some((BUTTON_2, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 18) => Some((BUTTON_3, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 19) => Some((BUTTON_4, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 20) => Some((BUTTON_5, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 21) => Some((BUTTON_6, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 22) => Some((BUTTON_7, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 23) => Some((BUTTON_8, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 24) => Some((BUTTON_9, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 25) => Some((BUTTON_10, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 26) => Some((BUTTON_11, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 27) => Some((BUTTON_12, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 28) => Some((BUTTON_13, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 29) => Some((BUTTON_14, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 30) => Some((BUTTON_15, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 31) => Some((BUTTON_16, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 32) => Some((BUTTON_17, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 33) => Some((BUTTON_18, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 34) => Some((BUTTON_19, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 35) => Some((BUTTON_20, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 36) => Some((BUTTON_21, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 37) => Some((BUTTON_22, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 38) => Some((BUTTON_23, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 39) => Some((BUTTON_24, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),

		// Flash buttons
		(1, 40) => Some((FADER_1, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Button(ControllerButtonEvent::Push { state, velocity: None })))),
		(1, 41) => Some((FADER_2, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Button(ControllerButtonEvent::Push { state, velocity: None })))),
		(1, 42) => Some((FADER_3, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Button(ControllerButtonEvent::Push { state, velocity: None })))),
		(1, 43) => Some((FADER_4, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Button(ControllerButtonEvent::Push { state, velocity: None })))),
		(1, 44) => Some((FADER_5, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Button(ControllerButtonEvent::Push { state, velocity: None })))),
		(1, 45) => Some((FADER_6, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Button(ControllerButtonEvent::Push { state, velocity: None })))),
		(1, 46) => Some((FADER_7, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Button(ControllerButtonEvent::Push { state, velocity: None })))),
		(1, 47) => Some((FADER_8, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Button(ControllerButtonEvent::Push { state, velocity: None })))),
		(1, 48) => Some((FADER_9, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Button(ControllerButtonEvent::Push { state, velocity: None })))),

		// Right buttons
		(1, 49) => Some((RIGHT_BUTTON_1, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 50) => Some((RIGHT_BUTTON_2, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 51) => Some((RIGHT_BUTTON_3, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 52) => Some((RIGHT_BUTTON_4, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 53) => Some((RIGHT_BUTTON_5, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		(1, 54) => Some((RIGHT_BUTTON_6, ControllerEvent::Button(ControllerButtonEvent::Push { state, velocity: None }))),
		_ => None
	}
}

pub fn create_controller_event(channel: u4, control: u7, value: u7) -> Option<(Uuid, ControllerEvent)> {
	match (channel.as_int(), control.as_int()) {
		// Faders
		(1, 1) => Some((FADER_1, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Pos(ScalableValue::U7(value)))))),
		(1, 2) => Some((FADER_2, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Pos(ScalableValue::U7(value)))))),
		(1, 3) => Some((FADER_3, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Pos(ScalableValue::U7(value)))))),
		(1, 4) => Some((FADER_4, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Pos(ScalableValue::U7(value)))))),
		(1, 5) => Some((FADER_5, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Pos(ScalableValue::U7(value)))))),
		(1, 6) => Some((FADER_6, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Pos(ScalableValue::U7(value)))))),
		(1, 7) => Some((FADER_7, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Pos(ScalableValue::U7(value)))))),
		(1, 8) => Some((FADER_8, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Pos(ScalableValue::U7(value)))))),
		(1, 9) => Some((FADER_9, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Pos(ScalableValue::U7(value)))))),

		// Fader touches
		(1, 101) => Some((FADER_1, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Touch(value.as_int() > 62))))),
		(1, 102) => Some((FADER_2, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Touch(value.as_int() > 62))))),
		(1, 103) => Some((FADER_3, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Touch(value.as_int() > 62))))),
		(1, 104) => Some((FADER_4, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Touch(value.as_int() > 62))))),
		(1, 105) => Some((FADER_5, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Touch(value.as_int() > 62))))),
		(1, 106) => Some((FADER_6, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Touch(value.as_int() > 62))))),
		(1, 107) => Some((FADER_7, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Touch(value.as_int() > 62))))),
		(1, 108) => Some((FADER_8, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Touch(value.as_int() > 62))))),
		(1, 109) => Some((FADER_9, ControllerEvent::FaderColumn(ControllerFaderColumnEvent::Fader(ControllerFaderEvent::Touch(value.as_int() > 62))))),

		// Knobs
		(1, 10) => Some((KNOB_1, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 11) => Some((KNOB_2, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 12) => Some((KNOB_3, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 13) => Some((KNOB_4, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 14) => Some((KNOB_5, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 15) => Some((KNOB_6, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 16) => Some((KNOB_7, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 17) => Some((KNOB_8, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 18) => Some((KNOB_9, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 19) => Some((KNOB_10, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 20) => Some((KNOB_11, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 21) => Some((KNOB_12, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 22) => Some((KNOB_13, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 23) => Some((KNOB_14, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 24) => Some((KNOB_15, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		(1, 25) => Some((KNOB_16, ControllerEvent::Knob(ControllerKnobEvent::Pos(ScalableValue::U7(value))))),
		_ => None,
	}
}

pub struct XTouchCompact {
	id: Uuid,
	meta: ControllerMeta,
	midi: MidiRouterInterface,
	midi_in: Uuid,
	midi_out: Uuid,
	controls: FxHashMap<Uuid, Arc<Control>>,
}
impl XTouchCompact {
	pub async fn new(
		meta: ControllerMeta,
		midi: MidiRouterInterface,
	) -> anyhow::Result<Self> {
		let controller_id = Uuid::new_v4();
		let mut controls = FxHashMap::default();

		let midi_in = midi.create_input(
			InputMeta {
				name: Arc::clone(&meta.name),
				group: Some(controller_id.clone()),
			},
			|packet| {
				if let Some((control, event)) = match LiveEvent::parse(&packet) {
					Ok(LiveEvent::Midi { channel, message }) => {
						match message {
							MidiMessage::NoteOff { key, vel } => create_note_event(false, channel, key),
							MidiMessage::NoteOn { key, vel } => create_note_event(true, channel, key),
							MidiMessage::Controller { controller, value } => create_controller_event(channel, controller, value),
							_ => None,
						}
					}
					_ => None,
				} {
					// Emit control, event
				}
			}
		).await;

		let midi_out = midi.create_output(OutputMeta {
			name: Arc::clone(&meta.name),
			group: Some(controller_id.clone()),
		}).await;

		return Ok(Self {
			id: controller_id,
			meta,
			midi_in,
			midi_out,
			midi,
			controls,
		});
	}
}
impl Controller for XTouchCompact {
	fn get_meta<'a>(&'a self) -> &'a ControllerMeta {
		return &self.meta;
	}
	fn get_controls<'a>(&'a self) -> &'a [ControlMeta] {
		return &CONTROLS;
	}
	fn get_control_by_uuid<'a>(&'a self, uuid: &Uuid) -> Option<Arc<Control>> {
		return self.controls.get(uuid).cloned();
	}
}


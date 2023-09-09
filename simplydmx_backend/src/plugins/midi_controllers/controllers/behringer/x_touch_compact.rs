use std::sync::Arc;

use async_trait::async_trait;
use midly::num::u4;
use rustc_hash::FxHashMap;
use uuid::{uuid, Uuid};
use serde::Deserialize;

use crate::{plugins::{
	live_controller::{
		control_proxies::AnalogToBoolean,
		scalable_value::ScalableValue,
		types::{
			ButtonControl, Control, ControlInstance, ControlState, Controller, FaderColumnControl,
			FaderControl, KnobControl,
		},
	},
	midi_controllers::controller_types::{MidiCC, MidiInterfaceController, MidiNote, MidiControllerProvider},
}, utilities::{forms::{FormDescriptor, NumberValidation}, serialized_data::SerializedData}};

async fn fader_column(
	controller: &mut MidiInterfaceController,
	name: &'static str,
	channel: u4,
	control: u8,
	touch: u8,
	flash: u8,
) -> ControlState {
	ControlState::new(Control::FaderColumn(ControlInstance::new(
		Arc::from(name),
		FaderColumnControl::build(
			FaderControl::build(
				MidiCC {
					recv_data: (channel.as_int(), control),
					send_data: Some((channel.as_int(), control)),
				}
				.create(controller)
				.unwrap(),
			)
			.with_touch(Arc::from(
				AnalogToBoolean::new(
					MidiCC {
						recv_data: (channel.as_int(), touch),
						send_data: Some((channel.as_int(), touch)),
					}
					.create(controller)
					.unwrap(),
					ScalableValue::U7(63.into()),
				)
				.await,
			)),
		)
		.with_flash_btn(ButtonControl::build(
			MidiNote {
				recv_data: (channel.as_int(), flash),
				send_data: Some((channel.as_int(), flash)),
			}
			.create(controller)
			.unwrap(),
		)),
	)))
}

async fn knob(
	controller: &mut MidiInterfaceController,
	name: &'static str,
	channel: u4,
	control: u8,
	push: u8,
) -> ControlState {
	ControlState::new(Control::Knob(ControlInstance::new(
		Arc::from(name),
		KnobControl::build(
			MidiCC {
				recv_data: (channel.as_int(), control),
				send_data: Some((channel.as_int(), control)),
			}
			.create(controller)
			.unwrap(),
		)
		.with_push(
			MidiNote {
				recv_data: (channel.as_int(), push),
				send_data: None,
			}
			.create(controller)
			.unwrap(),
		),
	)))
}

fn button(
	controller: &mut MidiInterfaceController,
	name: &'static str,
	channel: u4,
	push: u8,
) -> ControlState {
	ControlState::new(Control::Button(ControlInstance::new(
		Arc::from(name),
		ButtonControl::build(
			MidiNote {
				recv_data: (channel.as_int(), push),
				send_data: Some((channel.as_int(), push)),
			}
			.create(controller)
			.unwrap(),
		),
	)))
}

pub struct XTouchCompact;
#[async_trait]
impl MidiControllerProvider for XTouchCompact {
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
		return FormDescriptor::new().number_prefilled(
			"MIDI Channel",
			"channel",
			NumberValidation::And(vec![
				NumberValidation::Between(0.0, 15.0),
				NumberValidation::DivisibleBy(1.0),
			]),
			1.0,
		);
	}

	async fn create_controller(
		&self,
		form_data: SerializedData,
		controller: &mut MidiInterfaceController,
	) -> anyhow::Result<Controller> {
		#[derive(Deserialize, Debug)]
		struct XTouchCompactData {
			channel: u8,
		}

		let data: XTouchCompactData = form_data.deserialize()?;

		let built_controller = create_x_touch(controller, data.channel.into()).await;

		return Ok(built_controller);
	}
}


pub async fn create_x_touch(controller: &mut MidiInterfaceController, channel: u4) -> Controller {
	let controller = Controller {
		controls: FxHashMap::<Uuid, ControlState>::from_iter(
			[
				(
					uuid!("4663A89C-20F2-4CB8-909E-0BB07232EA02"),
					fader_column(controller, "Fader 1", channel, 1, 101, 40).await,
				),
				(
					uuid!("6E2F56B6-7D2E-4A4C-A851-B25C83E0CFCD"),
					fader_column(controller, "Fader 2", channel, 2, 102, 41).await,
				),
				(
					uuid!("7071C05B-AAE3-4558-B46D-0FB565DD8F54"),
					fader_column(controller, "Fader 3", channel, 2, 103, 42).await,
				),
				(
					uuid!("56F275BD-DAA9-4F8B-8FB1-18BCAC5D9EBA"),
					fader_column(controller, "Fader 4", channel, 2, 104, 43).await,
				),
				(
					uuid!("884310F0-2430-4BC1-9BE2-09012B793636"),
					fader_column(controller, "Fader 5", channel, 2, 105, 44).await,
				),
				(
					uuid!("A3A53834-EAFA-43D1-B4FE-867CD745B921"),
					fader_column(controller, "Fader 6", channel, 2, 106, 45).await,
				),
				(
					uuid!("12DA758E-D237-4E9E-8934-5F12596FC403"),
					fader_column(controller, "Fader 7", channel, 2, 107, 46).await,
				),
				(
					uuid!("B70E2E49-821C-4D83-8B4C-5B65AB04E11E"),
					fader_column(controller, "Fader 8", channel, 2, 108, 47).await,
				),
				(
					uuid!("5AD73FB0-83DE-487F-AE2E-BA9B6449EC72"),
					fader_column(controller, "Fader 9", channel, 2, 109, 48).await,
				),
				(
					uuid!("F3F153E8-9E32-4D05-A9E3-51C55E70B154"),
					knob(controller, "Knob 1", channel, 10, 0).await,
				),
				(
					uuid!("F4E1BBDD-8E30-4572-8D28-9F62E50B122A"),
					knob(controller, "Knob 2", channel, 11, 1).await,
				),
				(
					uuid!("A6DC2582-09E7-4E62-A8F9-70241BFB6126"),
					knob(controller, "Knob 3", channel, 12, 2).await,
				),
				(
					uuid!("A61DFAFE-4320-4FDF-956F-DF7204785B6F"),
					knob(controller, "Knob 4", channel, 13, 3).await,
				),
				(
					uuid!("A7581FC6-E12A-4AD6-A9EC-BAF49CF0A4C5"),
					knob(controller, "Knob 5", channel, 14, 4).await,
				),
				(
					uuid!("03A4CCCB-C2CE-4766-A5C3-806DA6608F5B"),
					knob(controller, "Knob 6", channel, 15, 5).await,
				),
				(
					uuid!("798C350D-891B-4754-B38A-F404B7C90E11"),
					knob(controller, "Knob 7", channel, 16, 6).await,
				),
				(
					uuid!("D010CE07-48F5-4EF1-9697-4680EADA4D97"),
					knob(controller, "Knob 8", channel, 17, 7).await,
				),
				(
					uuid!("D8792B3F-C89F-4E4A-B09C-4E428CA20940"),
					knob(controller, "Knob 9", channel, 18, 8).await,
				),
				(
					uuid!("E49E3A5A-D913-496F-91C6-C0A85465A5A8"),
					knob(controller, "Knob 10", channel, 19, 9).await,
				),
				(
					uuid!("DDCE924B-875F-4045-B454-FD3951357154"),
					knob(controller, "Knob 11", channel, 20, 10).await,
				),
				(
					uuid!("20CA1C7A-C6DD-4077-91F6-0B77CC491133"),
					knob(controller, "Knob 12", channel, 21, 11).await,
				),
				(
					uuid!("8E9848A9-B1EC-41CA-ABF8-A0F90F72C7A5"),
					knob(controller, "Knob 13", channel, 22, 12).await,
				),
				(
					uuid!("9C269774-1F69-4382-8A56-252A79D10AF6"),
					knob(controller, "Knob 14", channel, 23, 13).await,
				),
				(
					uuid!("0E458397-9E64-4990-81B0-D71BD2D62570"),
					knob(controller, "Knob 15", channel, 24, 14).await,
				),
				(
					uuid!("CF7A3BF8-80C5-4AA6-84C3-568976FDC565"),
					knob(controller, "Knob 16", channel, 25, 15).await,
				),
				(
					uuid!("B7E45052-A453-448F-9746-421209E04E5F"),
					button(controller, "Button 1", channel, 16),
				),
				(
					uuid!("1B9D3EF5-67DA-4153-AE9E-242853FDCC66"),
					button(controller, "Button 2", channel, 17),
				),
				(
					uuid!("886B75B7-7C1B-46B5-9390-BC003BC42D5E"),
					button(controller, "Button 3", channel, 18),
				),
				(
					uuid!("098C11AF-4DE9-4099-97A9-9F9E6D313980"),
					button(controller, "Button 4", channel, 19),
				),
				(
					uuid!("5AC69203-AD9E-4527-9E7C-7877327F9265"),
					button(controller, "Button 5", channel, 20),
				),
				(
					uuid!("C872ADDF-8B0A-4088-9DE4-D37C646ED838"),
					button(controller, "Button 6", channel, 21),
				),
				(
					uuid!("3546D405-D1E4-4CB8-B64F-71DE1B61A360"),
					button(controller, "Button 7", channel, 22),
				),
				(
					uuid!("DC5487F7-34F1-4A14-867D-1BA1BAA3B24B"),
					button(controller, "Button 8", channel, 23),
				),
				(
					uuid!("85156934-1182-45A8-ACCB-B5E36810BA5F"),
					button(controller, "Button 9", channel, 24),
				),
				(
					uuid!("929F6EBC-7630-48CF-9C27-443A7EE4A726"),
					button(controller, "Button 10", channel, 25),
				),
				(
					uuid!("7B8945FC-E390-4704-B062-B0B45EF850A1"),
					button(controller, "Button 11", channel, 26),
				),
				(
					uuid!("3463682F-6475-4F26-A07B-96840C136EFC"),
					button(controller, "Button 12", channel, 27),
				),
				(
					uuid!("E462A2E5-35A1-4FEB-8CCE-84B992B6B2D7"),
					button(controller, "Button 13", channel, 28),
				),
				(
					uuid!("BA7EA164-AC60-47FC-AB95-B86C67C03EFD"),
					button(controller, "Button 14", channel, 29),
				),
				(
					uuid!("F1216BE9-D327-4B42-AFF7-8C6ACE4B841B"),
					button(controller, "Button 15", channel, 30),
				),
				(
					uuid!("DF2DC765-3AA9-40FF-8A87-B35E5E34FD59"),
					button(controller, "Button 16", channel, 31),
				),
				(
					uuid!("53CA128D-3707-4257-BD24-9D79EE8ED7A3"),
					button(controller, "Button 17", channel, 32),
				),
				(
					uuid!("EC26F100-F943-4B35-AA2D-E98F742D5496"),
					button(controller, "Button 18", channel, 33),
				),
				(
					uuid!("DCF03D32-95EE-4340-B1F3-1E338349AD33"),
					button(controller, "Button 19", channel, 34),
				),
				(
					uuid!("6D745720-8682-4EE8-8D66-CC9ECE6DB2F7"),
					button(controller, "Button 20", channel, 35),
				),
				(
					uuid!("8D06DE65-93CA-4506-925B-46607C9B6CC5"),
					button(controller, "Button 21", channel, 36),
				),
				(
					uuid!("6BA3CE8F-E711-4D72-941A-3F4183F99551"),
					button(controller, "Button 22", channel, 37),
				),
				(
					uuid!("3B85FAAC-0D0D-4B13-9D62-83847163943A"),
					button(controller, "Button 23", channel, 38),
				),
				(
					uuid!("C0E5FCCE-A361-4C7D-B061-D5DC44F11BC2"),
					button(controller, "Button 24", channel, 39),
				),
				(
					uuid!("5C34B04D-3D8B-490A-8E46-4E25FE232B76"),
					button(controller, "Right button 1", channel, 49),
				),
				(
					uuid!("A7B05143-DBBC-4ED9-98A5-F1EFABFE2FA6"),
					button(controller, "Right button 2", channel, 50),
				),
				(
					uuid!("CF2D5E99-384C-4D81-8D48-33850E9F265B"),
					button(controller, "Right button 3", channel, 51),
				),
				(
					uuid!("A5A96F66-24C7-426E-96B6-90A935DAEB76"),
					button(controller, "Right button 4", channel, 52),
				),
				(
					uuid!("F3983309-E728-4797-BD5C-B651B839F1DE"),
					button(controller, "Right button 5", channel, 53),
				),
				(
					uuid!("41A23307-E1AD-4D2A-8D4D-C4477E997C4D"),
					button(controller, "Right button 6", channel, 54),
				),
			]
			.into_iter(),
		),
	};

	return controller;
}

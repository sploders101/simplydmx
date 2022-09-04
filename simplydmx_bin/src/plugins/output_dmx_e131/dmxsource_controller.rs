use std::{
	thread,
	collections::HashMap,
};
use async_std::{
	channel,
	task::block_on,
};

use sacn::DmxSource;

pub enum E131Command {
	CreateOutput,
	DestroyOutput,
	TerminateUniverse(u16),
	SendOutput(HashMap<u16, [u8; 512]>),
}

pub fn initialize_controller() -> channel::Sender<E131Command> {
	let (sender, receiver) = channel::bounded::<E131Command>(1);
	thread::spawn(move || {
		let mut controller: Option<DmxSource> = None;

		loop {
			let message = block_on(receiver.recv());
			if let Ok(message) = message {
				match message {
					E131Command::CreateOutput => {
						if controller.is_none() {
							let new_source = DmxSource::new("SimplyDMX");
							if let Ok(new_source) = new_source {
								controller = Some(new_source);
							} else {
								panic!("Could not start sACN controller");
							}
						}
					},
					E131Command::DestroyOutput => {
						controller = None;
					},
					E131Command::TerminateUniverse(universe_id) => {
						if let Some(ref controller) = controller {
							controller.terminate_stream(universe_id).ok();
						}
					},
					E131Command::SendOutput(output_data) => {
						if let Some(ref controller) = controller {
							for (universe_id, data) in output_data {
								controller.send(universe_id, &data).ok();
							}
						}
					},
				}
			} else {
				break;
			}
		}
	});
	return sender;
}

use async_std::{
	io,
	task,
	channel,
};
use async_std::prelude::*;

use crate::api_utilities::*;
use simplydmx_plugin_framework::*;

pub async fn initialize(plugin_context: PluginContext) {

	// Channels
	let (request_sender, request_receiver) = channel::unbounded();
	let (response_sender, response_receiver) = channel::unbounded();

	spawn_api_facet_controller(plugin_context.clone(), request_receiver, response_sender);

	// Receiver
	task::spawn(async move {
		let stdin = io::stdin();
		loop {
			let mut line = String::new();
			if let Ok(read_length) = stdin.read_line(&mut line).await {
				if read_length > 0 {
					if let Ok(command) = serde_json::from_str::<JSONCommand>(&line) {
						request_sender.send(command).await.unwrap();
					} else {
						log_error!(plugin_context, "Discarded unrecognized command: {}", line);
					}
				}
			} else {
				break;
			}
		}

		log!(plugin_context, "API host on stdio stopped.");
	});

	// Responder
	task::spawn(async move {
		loop {
			let message = response_receiver.recv().await;
			if let Ok(message) = message {
				let mut stdout = io::stdout();
				if let Ok(mut data) = serde_json::to_vec(&message) {
					data.push(b"\n"[0]);
					stdout.write_all(&data).await.ok();
				}
			} else {
				break;
			}
		}
	});

}

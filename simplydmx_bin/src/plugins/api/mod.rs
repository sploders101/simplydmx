use async_std::{
	io,
	task,
};
use async_std::prelude::*;

use simplydmx_plugin_framework::*;

#[portable]
#[serde(tag = "type")]
pub enum JSONCommand {

	// Services
	CallService {
		message_id: u32,
		plugin_id: String,
		service_id: String,
		args: Vec<serde_json::Value>,
	},
	GetServices {
		message_id: u32,
	},

	// Events
	SendEvent {
		name: String,
		data: serde_json::Value,
	},
	Subscribe {
		name: String,
	},
	Unsubscribe {
		name: String,
	},

}

#[portable]
#[serde(tag = "type")]
pub enum JSONResponse {

	CallServiceResponse {
		message_id: u32,
		result: serde_json::Value,
	},

	ServiceList {
		message_id: u32,
		// list: Vec<String>,
	},

	CallServiceError {
		message_id: u32,
		error: JSONCallServiceError,
	}

}

#[portable]
#[serde(tag = "type")]
pub enum JSONCallServiceError {
	ServiceNotFound,
	ArgDeserializationFailed,
	ResponseSerializationFailed,
}

pub async fn initialize(plugin_context: PluginContext) {

	// Start server
	stdio_controller(plugin_context);

}

fn stdio_controller(plugin_context: PluginContext) {
	task::spawn(async move {
		let stdin = io::stdin();
		loop {
			let mut line = String::new();
			if let Ok(read_length) = stdin.read_line(&mut line).await {
				if read_length > 0 {
					if let Ok(command) = serde_json::from_str::<JSONCommand>(&line) {
						handle_command(plugin_context.clone(), command);
					} else {
						call_service!(plugin_context, "core", "log", format!("Discarded unrecognized command: {}", line));
					}
				}
			} else {
				break;
			}
		}

		call_service!(plugin_context, "core", "log", String::from("API host on stdio stopped."));
	});
}

fn handle_command(plugin_context: PluginContext, command: JSONCommand) {
	task::spawn(async move {
		match command {

			JSONCommand::CallService { message_id, plugin_id, service_id, args } => {
				let result = plugin_context.get_service(&plugin_id, &service_id).await;
				match result {
					Ok(service) => {
						match service.call_json(args).await {
							Ok(result) => {
								send_response(JSONResponse::CallServiceResponse {
									message_id,
									result,
								}).await;
							},
							Err(error) => {
								send_response(JSONResponse::CallServiceError {
									message_id,
									error: match error {
										CallServiceJSONError::DeserializationFailed => JSONCallServiceError::ArgDeserializationFailed,
										CallServiceJSONError::SerializationFailed => JSONCallServiceError::ResponseSerializationFailed,
									},
								}).await;
							},
						}
					},
					Err(_) => {
						send_response(JSONResponse::CallServiceError {
							message_id,
							error: JSONCallServiceError::ServiceNotFound,
						}).await;
					}
				}
			},

			JSONCommand::GetServices { message_id } => {},
			JSONCommand::SendEvent { name, data } => {},
			JSONCommand::Subscribe { name } => {},
			JSONCommand::Unsubscribe { name } => {},
		}
	});
}

async fn send_response(message: JSONResponse) {
	let mut stdout = io::stdout();
	if let Ok(data) = serde_json::to_vec(&message) {
		stdout.write_all(&data).await.ok();
	}
}

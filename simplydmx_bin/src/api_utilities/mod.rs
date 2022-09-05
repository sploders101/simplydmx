mod events;
use events::EventJuggler;

use async_std::{
	task,
	channel,
};

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
		criteria: Option<FilterCriteria>,
		data: serde_json::Value,
	},
	Subscribe {
		name: String,
		criteria: Option<FilterCriteria>,
	},
	Unsubscribe {
		name: String,
		criteria: Option<FilterCriteria>,
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
		list: Vec<ServiceDescription>,
	},

	CallServiceError {
		message_id: u32,
		error: JSONCallServiceError,
	},

	Event {
		name: String,
		criteria: FilterCriteria,
		data: serde_json::Value,
	},

}

#[portable]
#[serde(tag = "type")]
pub enum JSONCallServiceError {
	ServiceNotFound,
	ArgDeserializationFailed,
	ResponseSerializationFailed,
}

pub fn spawn_api_facet_controller(plugin_context: PluginContext, receiver: channel::Receiver<JSONCommand>, sender: channel::Sender<JSONResponse>) {
	// Receiver
	task::spawn(async move {
		let juggler = EventJuggler::new(&plugin_context, sender.clone());
		loop {
			if let Ok(command) = receiver.recv().await {
				handle_command(plugin_context.clone(), command, juggler.clone(), sender.clone());
			} else {
				break;
			}
		}

		log!(plugin_context, "API host on stdio stopped.");
	});
}

fn handle_command(plugin_context: PluginContext, command: JSONCommand, juggler: EventJuggler, sender: channel::Sender<JSONResponse>) {
	task::spawn(async move {
		match command {

			JSONCommand::CallService { message_id, plugin_id, service_id, args } => {
				#[cfg(feature = "verbose-debugging")]
				println!("Getting {}.{}", &plugin_id, &service_id);
				let result = plugin_context.get_service(&plugin_id, &service_id).await;
				#[cfg(feature = "verbose-debugging")]
				println!("Got {}.{}", &plugin_id, &service_id);
				match result {
					Ok(service) => {
						#[cfg(feature = "verbose-debugging")]
						println!("Calling {}.{} with {:?}", &plugin_id, service_id, args);
						match service.call_json(args).await {
							Ok(result) => {
								#[cfg(feature = "verbose-debugging")]
								println!("Sending response to {}.{}", &plugin_id, &service_id);
								sender.send(JSONResponse::CallServiceResponse {
									message_id,
									result,
								}).await.ok();
							},
							Err(error) => {
								#[cfg(feature = "verbose-debugging")]
								println!("Sending response to {}.{}", &plugin_id, &service_id);
								sender.send(JSONResponse::CallServiceError {
									message_id,
									error: match error {
										CallServiceRPCError::DeserializationFailed => JSONCallServiceError::ArgDeserializationFailed,
										CallServiceRPCError::SerializationFailed => JSONCallServiceError::ResponseSerializationFailed,
									},
								}).await.ok();
							},
						}
						#[cfg(feature = "verbose-debugging")]
						println!("Call to {}.{} finished", &plugin_id, &service_id);
					},
					Err(_) => {
						sender.send(JSONResponse::CallServiceError {
							message_id,
							error: JSONCallServiceError::ServiceNotFound,
						}).await.ok();
					}
				}
			},

			JSONCommand::GetServices { message_id } => {
				sender.send(JSONResponse::ServiceList {
					message_id,
					list: plugin_context.list_services().await,
				}).await.ok();
			},
			JSONCommand::SendEvent { name, criteria, data } => {
				plugin_context.emit_json(name, criteria.unwrap_or(FilterCriteria::None), data).await;
			},
			JSONCommand::Subscribe { name, criteria } => {
				juggler.add_event_listener(name, criteria.unwrap_or(FilterCriteria::None)).await.ok();
			},
			JSONCommand::Unsubscribe { name, criteria } => {
				juggler.remove_event_listener(name, criteria.unwrap_or(FilterCriteria::None)).await;
			},
		}
	});
}

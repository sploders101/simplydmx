mod events;
use events::EventJuggler;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use simplydmx_plugin_framework::*;

#[portable]
#[serde(tag = "type")]
/// Describes a command to be sent via a JSON or equivalent API
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

	// Options provider
	GetOptions {
		message_id: u32,
		provider_id: String,
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
/// Describes an event to be sent to a client via a JSON or equivalent API
pub enum JSONResponse {

	CallServiceResponse {
		message_id: u32,
		result: serde_json::Value,
	},

	ServiceList {
		message_id: u32,
		list: Vec<ServiceDescription>,
	},

	OptionsList {
		message_id: u32,
		list: Result<Vec<DropdownOptionJSON>, TypeSpecifierRetrievalError>,
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
/// An error that could occur while attempting to call a JSON service
pub enum JSONCallServiceError {
	ServiceNotFound,
	ArgDeserializationFailed,
	ResponseSerializationFailed,
}

pub async fn spawn_api_facet_controller(plugin_context: PluginContext, mut receiver: UnboundedReceiver<JSONCommand>, sender: UnboundedSender<JSONResponse>) {
	// Receiver
	let mut shutdown_receiver = plugin_context.on_shutdown().await;
	plugin_context.clone().spawn_volatile("API Facet Controller", async move {
		let juggler = EventJuggler::new(plugin_context.clone(), sender.clone());
		loop {
			tokio::select! {
				command = receiver.recv() => {
					if let Some(command) = command {
						handle_command(plugin_context.clone(), command, juggler.clone(), sender.clone()).await;
					} else {
						break;
					}
				},
				_ = shutdown_receiver.recv() => break,
			}
		}

		log!(plugin_context, "API host has stopped.");
	}).await;
}

async fn handle_command(plugin_context: PluginContext, command: JSONCommand, juggler: EventJuggler, sender: UnboundedSender<JSONResponse>) {
	plugin_context.clone().spawn_volatile("API Command Runner", async move {
		match command {

			JSONCommand::CallService { message_id, plugin_id, service_id, args } => {
				let result = plugin_context.get_service(&plugin_id, &service_id).await;
				match result {
					Ok(service) => {
						match service.call_json(args).await {
							Ok(result) => {
								sender.send(JSONResponse::CallServiceResponse {
									message_id,
									result,
								}).ok();
							},
							Err(error) => {
								sender.send(JSONResponse::CallServiceError {
									message_id,
									error: match error {
										CallServiceRPCError::DeserializationFailed => JSONCallServiceError::ArgDeserializationFailed,
										CallServiceRPCError::SerializationFailed => JSONCallServiceError::ResponseSerializationFailed,
									},
								}).ok();
							},
						}
					},
					Err(_) => {
						sender.send(JSONResponse::CallServiceError {
							message_id,
							error: JSONCallServiceError::ServiceNotFound,
						}).ok();
					}
				}
			},

			JSONCommand::GetServices { message_id } => {
				sender.send(JSONResponse::ServiceList {
					message_id,
					list: plugin_context.list_services().await,
				}).ok();
			},

			JSONCommand::GetOptions { message_id, provider_id } => {
				sender.send(JSONResponse::OptionsList {
					message_id,
					list: plugin_context.get_service_type_options_json(&provider_id).await,
				}).ok();
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
	}).await;
}

use async_std::{
	task,
	channel,
};
use tauri::Manager;
use crate::api_utilities::*;
use simplydmx_plugin_framework::*;


#[tauri::command]
async fn sdmx(sender: tauri::State<'_, channel::Sender<JSONCommand>>, message: JSONCommand) -> Result<(), &'static str> {
	match sender.send(message).await {
		Ok(_) => { return Ok(()); },
		Err(_) => {
			return Err("Could not communicate with SimplyDMX API.");
		},
	};
}


/// WARNING!!! This function blocks the thread indefinitely!
///
/// Run async tasks in another thread to avoid blocking the application.
///
/// This initializes the graphical interface and creates a communication channel with SimplyDMX's
/// JSON API service. Tauri-specific functions are only used as a connector to SimplyDMX's built-in
/// APIs so that the UI has full access to all features and remains framework-agnostic.
pub fn initialize(plugin_context: PluginContext) {

	// Channels
	let (request_sender, request_receiver) = channel::unbounded();
	let (response_sender, response_receiver) = channel::unbounded();

	spawn_api_facet_controller(plugin_context.clone(), request_receiver, response_sender);

	tauri::Builder::default()
		.manage(request_sender)
		.invoke_handler(tauri::generate_handler![sdmx])
		.setup(move |app| {
			let app = app.app_handle();
			task::spawn(async move {
				loop {
					if let Ok(msg) = response_receiver.recv().await {
						app.emit_all("sdmx", msg).ok();
					} else {
						break;
					}
				}
			});

			return Ok(());
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");

	println!("Tauri call completed");

}

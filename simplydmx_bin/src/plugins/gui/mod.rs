use std::sync::Arc;

use async_std::{
	task,
	channel,
	sync::RwLock,
};
use tauri::{
	Manager,
	async_runtime,
	WindowEvent,
};
use crate::{
	api_utilities::*,
	async_main,
};
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
pub async fn initialize(plugin_manager: PluginManager) {

	let shutdown_receiver = plugin_manager.on_shutdown().await;

	// Create the GUI plugin context
	let plugin_context = task::block_on(plugin_manager.register_plugin(
		"gui",
		"SimplyDMX GUI",
	)).unwrap();

	// Spawn the rest of SimplyDMX in a new thread
	let plugin_manager_copy = plugin_manager.clone();
	task::block_on(async_main(plugin_manager_copy));

	// Channels
	let (request_sender, request_receiver) = channel::unbounded();
	let (response_sender, response_receiver) = channel::unbounded();

	spawn_api_facet_controller(plugin_context.clone(), request_receiver, response_sender).await;


	let shutting_down = Arc::new(RwLock::new(false));


	let plugin_manager_win_evt = plugin_manager.clone();
	tauri::Builder::default()
		.manage(request_sender)
		.invoke_handler(tauri::generate_handler![sdmx])
		.on_window_event(move |event| match event.event() {
			WindowEvent::CloseRequested { api, .. } => {

				// Prevent close
				api.prevent_close();


				// Issue shutdown
				let plugin_manager_win_evt = plugin_manager_win_evt.clone();
				let shutting_down = Arc::clone(&shutting_down);
				async_runtime::spawn(async move {
					let mut shutting_down = shutting_down.write().await;
					if *shutting_down {
						return;
					}
					*shutting_down = true;
					drop(shutting_down);
					plugin_manager_win_evt.shutdown().await;
				});

			},
			_ => {},
		})
		.setup(move |app_ref| {
			let app = app_ref.app_handle();
			async_runtime::spawn(async move {
				loop {
					if let Ok(msg) = response_receiver.recv().await {
						app.emit_all("sdmx", msg).ok();
					} else {
						break;
					}
				}
			});

			let app = app_ref.app_handle();
			async_runtime::spawn(async move {
				// Wait for shutdown request
				shutdown_receiver.recv().await.unwrap();

				// Finish shutdown
				plugin_manager.finish_shutdown().await;

				#[cfg(feature = "verbose-debugging")]
				println!("Successfully shut down.");

				for window in app.windows().values() {
					window.close().unwrap();
				}
			});

			return Ok(());
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");

	#[cfg(feature = "verbose-debugging")]
	println!("Tauri call completed");

}

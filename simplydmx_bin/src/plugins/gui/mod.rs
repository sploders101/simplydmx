use std::sync::Arc;

use async_std::{
	task::block_on,
	channel,
	sync::RwLock,
};
use futures::{
	select,
	FutureExt,
};
use tauri::{
	Manager,
	AppHandle,
	RunEvent,
};
use crate::{
	api_utilities::*,
	init::async_main,
};
use simplydmx_plugin_framework::*;


/// Holds the state of the application in a data structure that can be swapped out when the system is reloaded
struct ApplicationState {
	plugin_manager: PluginManager,
	api_sender: channel::Sender<JSONCommand>,
}
impl ApplicationState {
	async fn start_plugins(app: AppHandle, file: Option<Vec<u8>>) -> Self {
		let manager = PluginManager::new();
		let plugin = manager.register_plugin("gui", "Tauri UI").await.unwrap();

		// Boot up SimplyDMX
		async_main(manager.clone(), file).await;

		// API Setup
		let (request_sender, request_receiver) = channel::unbounded();
		let (response_sender, response_receiver) = channel::unbounded();
		spawn_api_facet_controller(plugin.clone(), request_receiver, response_sender).await;

		// Response channel setup
		let shutdown_receiver = plugin.on_shutdown().await;
		plugin.spawn_volatile("GUI API Responder", async move {
			loop {
				select! {
					msg = response_receiver.recv().fuse() => {
						if let Ok(msg) = msg {
							app.emit_all("sdmx", msg).ok();
						} else {
							break;
						}
					},
					_ = shutdown_receiver.recv().fuse() => break,
				}
			}
		}).await;

		return ApplicationState {
			plugin_manager: manager,
			api_sender: request_sender,
		};
	}
}


#[tauri::command]
async fn sdmx(state: tauri::State<'_, Arc<RwLock<Option<ApplicationState>>>>, message: JSONCommand) -> Result<(), &'static str> {
	if let Some(ref app_state) = *state.read().await {
		match app_state.api_sender.send(message).await {
			Ok(_) => { return Ok(()); },
			Err(_) => {
				return Err("Could not communicate with SimplyDMX API.");
			},
		};
	} else {
		return Err("SimplyDMX not finished initializing");
	}
}

#[tauri::command]
async fn load_file(app: tauri::AppHandle, state: tauri::State<'_, Arc<RwLock<Option<ApplicationState>>>>, file: Option<Vec<u8>>) -> Result<(), &'static str> {
	let mut writable_state = state.write().await;

	// Shut down existing instance
	if let Some(ref state) = *writable_state {
		state.plugin_manager.shutdown().await;
		state.plugin_manager.finish_shutdown().await;
	}

	// Reload
	*writable_state = Some(ApplicationState::start_plugins(app, file).await);

	return Ok(());
}


/// WARNING!!! This function blocks the thread indefinitely!
///
/// This should be the only function running from `fn main()`. It sets up the GUI, starts SimplyDMX's
/// plugin system, and controls save/load.
///
/// This initializes the graphical interface and creates a communication channel with SimplyDMX's
/// JSON API service. Tauri-specific functions are only used as a connector to SimplyDMX's built-in
/// APIs so that the UI has full access to all features and remains framework-agnostic.
pub async fn initialize() {

	let quitting = Arc::new(RwLock::new(false));

	let application_state: Arc<RwLock<Option<ApplicationState>>> = Arc::new(RwLock::new(None));
	let application_state_setup = Arc::clone(&application_state);
	let application_state_exit = Arc::clone(&application_state);

	tauri::Builder::default()
		.manage(application_state)
		.invoke_handler(tauri::generate_handler![sdmx, load_file])
		.setup(move |app_ref| {
			let app = app_ref.app_handle();
			let mut application_state = block_on(application_state_setup.write());
			*application_state = Some(block_on(ApplicationState::start_plugins(app, None)));

			return Ok(());
		})
		.build(tauri::generate_context!()).expect("error while running tauri application")
		.run(move |app_handle, event| match event {
			RunEvent::Exit => {
				// Issue shutdown if necessary
				let application_state = Arc::clone(&application_state_exit);
				let quitting = Arc::clone(&quitting);
				let app_handle = app_handle.clone();
				// If we're already quitting, ignore the event. Otherwise, mark that we are quitting
				let mut quitting = block_on(quitting.write());
				if *quitting {
					return;
				}
				*quitting = true;
				drop(quitting);

				// If we have a plugin system, shut it down. Otherwise, go ahead and close the window
				if let Some(ref state) = *block_on(application_state.read()) {
					block_on(state.plugin_manager.shutdown());
					block_on(state.plugin_manager.finish_shutdown());
					#[cfg(feature = "verbose-debugging")]
					println!("Successfully shut down.");
				}
				app_handle.exit(0);
			},
			_ => {},
		});

	#[cfg(feature = "verbose-debugging")]
	println!("Tauri call completed");

}

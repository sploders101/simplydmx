use std::thread;
use async_std::{
	io,
	task,
	channel,
};
use async_std::prelude::*;

use crate::api_utilities::*;
use simplydmx_plugin_framework::*;

pub async fn initialize(plugin_context: PluginContext) {

	tauri::Builder::default()
		// .invoke_handler(tauri::generate_handler![greet])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");

	println!("Tauri call completed");

}

#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

pub mod plugins;
pub mod api_utilities;
pub mod mixer_utils;
pub mod utilities;
pub mod init;

use async_std::task;

fn main() {

	#[cfg(feature = "gui")]
	task::block_on(plugins::gui::initialize());

}

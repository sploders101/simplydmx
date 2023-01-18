#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

pub mod plugins;
pub mod api_utilities;
pub mod mixer_utils;
pub mod utilities;
pub mod init;

fn main() {

	#[cfg(all(feature = "export-services", feature = "gui"))]
	compile_error!("export-services cannot be used with an application runtime. Please remove the runtime feature (eg. `gui`) or use `--no-default-features`");

	#[cfg(all(feature = "export-services", not(debug_assertions)))]
	compile_error!("export-services cannot be used in release mode. The lack of a runtime means that not all types are included in release mode since they don't get used.");

	#[cfg(feature = "export-services")]
	init::exporter::rpc_coverage();

}

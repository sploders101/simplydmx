use simplydmx_plugin_framework::*;
use std::collections::HashMap;

#[portable]
/// Describes an entire show file containing fragments from every plugin
pub struct ShowFile {
	pub plugin_data: HashMap<String, Vec<u8>>,
}

use std::collections::HashMap;
use simplydmx_plugin_framework::*;

#[portable]
pub struct ShowFile {
	pub plugin_data: HashMap<String, Vec<u8>>,
}

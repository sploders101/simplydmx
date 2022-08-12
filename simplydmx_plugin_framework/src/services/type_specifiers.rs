use std::any::Any;
use serde_json::Value;
use simplydmx_plugin_macros::portable;

/// This trait provides an interface for querying potential options
pub trait TypeSpecifier {
	fn is_exclusive(&self) -> bool;
	fn get_options(&self) -> Vec<DropdownOptionNative>;
	fn get_options_json(&self) -> Vec<DropdownOptionJSON>;
}

pub struct DropdownOptionNative {
	pub name: String,
	pub description: Option<String>,
	pub value: Box<dyn Any>,
}

#[portable]
pub struct DropdownOptionJSON {
	pub name: String,
	pub description: Option<String>,
	pub value: Value,
}

use std::any::Any;
use serde_json::Value;
use async_trait::async_trait;
use simplydmx_plugin_macros::portable;

/// This trait provides an interface for querying potential options
#[async_trait]
pub trait TypeSpecifier {
	async fn get_options(&self) -> Vec<DropdownOptionNative>;
	async fn get_options_json(&self) -> Vec<DropdownOptionJSON>;
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

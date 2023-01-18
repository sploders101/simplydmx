use serde_json::Value;
use async_trait::async_trait;
use simplydmx_plugin_macros::portable;

use crate::PortableMessage;

/// This trait provides an interface for querying potential options
#[async_trait]
pub trait TypeSpecifier {
	async fn get_options(&self) -> Vec<DropdownOptionNative>;
}

pub struct DropdownOptionNative {
	pub name: String,
	pub description: Option<String>,
	pub value: Box<dyn 'static + PortableMessage + Sync + Send>,
}

#[portable]
pub struct DropdownOptionJSON {
	pub name: String,
	pub description: Option<String>,
	pub value: Value,
}

impl TryFrom<DropdownOptionNative> for DropdownOptionJSON {
	type Error = serde_json::Error;
	fn try_from(value: DropdownOptionNative) -> Result<Self, Self::Error> {
		return Ok(DropdownOptionJSON {
			name: value.name,
			description: value.description,
			value: value.value.serialize_json()?,
		});
	}
}

use simplydmx_plugin_framework::*;
use uuid::Uuid;
use serde_big_array::BigArray;

/// Descriptor indicating parameters for communicating with a DMX driver
///
/// Drivers are implemented as a collection of services, referenced by this descriptor.
#[portable]
pub struct DMXDriverDescriptor {

	/// The unique ID of the DMX driver
	pub id: String,

	/// The human-readable name of the DMX driver
	pub name: String,

	/// A human-readable description of the driver, such as what devices and protocols it uses
	pub description: String,

	/// The plugin ID of the DMX driver
	pub plugin_id: String,

	/// The ID of the "Register Universe" service
	pub register_universe_service: String,

	/// The ID of the "Delete Universe" service
	pub delete_universe_service: String,

	/// The ID of the "Send DMX Frame" service
	pub output_service: String,

}

/// Minified representation of a DMX driver for display
#[portable]
pub struct DisplayableDMXDriver {
	pub id: String,
	pub name: String,
	pub description: String,
}

/// "Portable",  representation of a DMX frame that implements Into<[u8; 512]>
#[portable]
pub struct DMXFrame {
	pub id: Uuid,
	#[serde(with = "BigArray")]
	pub frame: [u8; 512],
}

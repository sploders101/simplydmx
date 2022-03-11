use std::sync::Arc;

use serde_json::json;
use async_std::test;

// Create an alias for macro output to use since this is an internal function
// and the macro outputs fully-qualified type paths
mod simplydmx_plugin_framework {
	pub use crate::*;
}

use crate::{
	Service,
	interpolate_service,
};

struct Test ();
#[interpolate_service(
	Test,
	"test_service",
	"Test Service",
	"Test service's description"
)]
impl TestService {
	#[service_main(
		("Called From", "This indicates how the service was called."),
		("Light", "This is the ID of the light that you would like to control"),
		("New Value", "This is the value you want to assign to the light (0-65535)"),
		("Misc Values", "These are some more miscellaneous values to show off the service framework", "some-stuff"),
		("Formatted string", "This is a formatted string created from the inputs that were supplied"),
	)]
	pub fn call_internal(self, from: String, light_id: u32, value: Option::<u16>, values: Vec::<String>) -> String {
		// Do stuff here
		return format!("From {}: Set light {:?} to {:?}. Here are some misc values: {:?}", from, light_id, value, values);
	}
}

#[test]
async fn smoke_test() {

	// Create an instance of TestService
	let service = TestService (Arc::new(Test ()));

	// Convert TestService to generic Service trait implementation
	let service: Box<dyn Service> = Box::new(service);

	// Call TestService as a generic Service trait with native values
	let service_result = service.call(vec![
		Box::new(String::from("Native Values")),
		Box::new(15u32),
		Box::new(Some(65535u16)),
		Box::new(vec![
			String::from("Value 1"),
			String::from("Value 2"),
			String::from("Value 3"),
			String::from("Value 4")
		])
	]).await.unwrap();
	assert_eq!(
		service_result.downcast_ref::<String>().expect("Service result was not a string"),
		r#"From Native Values: Set light 15 to Some(65535). Here are some misc values: ["Value 1", "Value 2", "Value 3", "Value 4"]"#
	);

	// Call TestService as a generic Service trait with JSON values
	let service_result = service.call_json(serde_json::from_str(r#"[
		"JSON",
		15,
		65535,
		[
			"Value 1",
			"Value 2",
			"Value 3",
			"Value 4"
		]
	]"#).expect("Couldn't parse JSON input")).await.unwrap();
	assert_eq!(
		service_result,
		json!(r#"From JSON: Set light 15 to Some(65535). Here are some misc values: ["Value 1", "Value 2", "Value 3", "Value 4"]"#)
	);

	// Check metadata supplied by TestService through the Service trait
	assert_eq!(service.get_id(), "test_service",);
	assert_eq!(service.get_name(), "Test Service",);
	assert_eq!(service.get_description(), "Test service's description");

	// Check that the function signature is correct and serializes appropriately
	let service_description = service.get_signature();
	assert_eq!(
		json!({
			"args": service_description.0,
			"return": service_description.1
		}),
		json!({
			"args": [
				{
					"description": "This indicates how the service was called.",
					"id": "from",
					"name": "Called From",
					"val_type": {
						"modifier": "Required",
						"value": {
							"type": "String"
						}
					},
					"val_type_id": null
				},
				{
					"description": "This is the ID of the light that you would like to control",
					"id": "light_id",
					"name": "Light",
					"val_type": {
						"modifier": "Required",
						"value": {
							"type": "U32"
						}
					},
					"val_type_id": null
				},
				{
					"description": "This is the value you want to assign to the light (0-65535)",
					"id": "value",
					"name": "New Value",
					"val_type": {
						"modifier": "Optional",
						"value": {
							"type": "U16"
						}
					},
					"val_type_id": null
				},
				{
					"description": "These are some more miscellaneous values to show off the service framework",
					"id": "values",
					"name": "Misc Values",
					"val_type": {
						"modifier": "Vector",
						"value": {
							"type": "String"
						}
					},
					"val_type_id": "some-stuff"
				}
			],
			"return": {
				"description": "This is a formatted string created from the inputs that were supplied",
				"id": "return",
				"name": "Formatted string",
				"val_type": {
					"modifier": "Required",
					"value": {
						"type": "String"
					}
				},
				"val_type_id": null
			}
		})
	);

}

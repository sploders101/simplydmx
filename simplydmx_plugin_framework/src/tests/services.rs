use std::{
	sync::Arc,
	time::Duration,
};

use serde_json::json;
use tokio::time::sleep;

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
	"test_service",
	"Test Service",
	"Test service's description"
)]
impl TestService {
	#![inner(Test)]
	#[service_main(
		"This indicates how the service was called.",
		"This is the ID of the light that you would like to control",
		"This is the value you want to assign to the light (0-65535)",
		("These are some more miscellaneous values to show off the service framework", "some-stuff"),
		"This is a formatted string created from the inputs that were supplied",
	)]
	pub async fn call_internal(self, from: String, light_id: u32, value: Option::<u16>, values: Vec::<String>) -> String {
		// Check carrying of data across await
		sleep(Duration::from_millis(5)).await;
		return format!("From {}: Set light {:?} to {:?}. Here are some misc values: {:?}", from, light_id, value, values);
	}
}

#[tokio::test]
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
					"val_type": "String",
					"val_type_hint": null
				},
				{
					"description": "This is the ID of the light that you would like to control",
					"id": "light_id",
					"val_type": "u32",
					"val_type_hint": null
				},
				{
					"description": "This is the value you want to assign to the light (0-65535)",
					"id": "value",
					"val_type": "Option :: < u16 >",
					"val_type_hint": null
				},
				{
					"description": "These are some more miscellaneous values to show off the service framework",
					"id": "values",
					"val_type": "Vec :: < String >",
					"val_type_hint": "some-stuff"
				}
			],
			"return": {
				"description": "This is a formatted string created from the inputs that were supplied",
				"id": "return",
				"val_type": "String",
				"val_type_hint": null
			}
		})
	);

}

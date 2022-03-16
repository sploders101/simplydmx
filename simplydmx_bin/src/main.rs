use serde_json::json;
use serde_json::Value;

use simplydmx_plugin_framework::{
	Service,
	interpolate_service,
};

use std::{
	sync::Arc,
	boxed::Box,
	time::Duration,
};

use async_std::task::sleep;

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
	pub async fn call_internal(self, from: String, light_id: u32, value: Option::<u16>, values: Vec::<String>) -> String {
		// Do stuff here
		sleep(Duration::from_secs(1)).await;
		return format!("From {}: Set light {:?} to {:?}. Here are some misc values: {:?}", from, light_id, value, values);
	}
}

#[async_std::main]
async fn main() {
	// Create an instance of TestService
	let service = TestService (Arc::new(Test ()));

	// Erase all specifics about TestService
	let service: Box<dyn Service> = Box::new(service);

	// Call TestService as a generic Service trait
	println!("Running with native values...");
	let native_result = service.call(vec![
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
	let native_result = native_result.downcast_ref::<String>().expect("Result should be a string");
	println!("{}", native_result);

	println!("Running with JSON values...");
	let json_result = service.call_json(serde_json::from_str(r#"[
		"JSON_________",
		15,
		65535,
		[
			"Value 1",
			"Value 2",
			"Value 3",
			"Value 4"
		]
	]"#).unwrap()).await.unwrap();
	if let Value::String(string) = json_result {
		println!("{}", string);
	} else {
		panic!("Result should be a string");
	}

	// Give the user a chance to read the async service results
	sleep(Duration::from_secs(1)).await;

	// Print information supplied by TestService through the Service trait
	println!("ID:          {}", service.get_id());
	println!("Name:        {}", service.get_name());
	println!("Description: {}", service.get_description());

	// Print the function signature of TestService as JSON
	let service_description = service.get_signature();
	println!("{}", serde_json::to_string_pretty(&json!({
		"args": service_description.0,
		"return": service_description.1
	})).unwrap());
}

use std::any::Any;
use serde::{
	Serialize,
	Deserialize,
};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceArgument {
	name: String,
	description: String,
	val_type: ServiceArgumentModifiers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "modifier", content = "value")]
pub enum ServiceArgumentModifiers {
	Required(ServiceDataTypes),
	Optional(ServiceDataTypes),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "desc")]
pub enum ServiceDataTypes {
	U8,
	U16,
	U32,
	I8,
	I16,
	I32,
	String,

	/// This data type allows a service to specify its own type. Its argument
	/// should contain a description of the custom format.
	Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CallServiceError {
	TypeValidationFailed,
}

pub trait Service {

	/// Gets the name of the service
	fn get_name<'a>(&'a self) -> &'a str;

	/// Gets a description indicating what the service does
	fn get_description<'a>(&'a self) -> &'a str;

	/// Get the documentation for the arguments required by the service
	fn get_signature<'a>(&'a self) -> (&[ServiceArgument], Option<ServiceArgument>);

	/// Call the service locally without static typing
	fn call(&self, arguments: Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, CallServiceError>;

	/// Call the service using JSON values
	fn call_json(&self, arguments: Vec<Value>) -> Result<Value, CallServiceError>;
}

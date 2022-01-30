use std::any::Any;
use serde::{
	Serialize,
	Deserialize,
};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceArgument {
	pub id: String,
	pub name: String,
	pub description: String,
	pub val_type: ServiceArgumentModifiers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "modifier", content = "value")]
pub enum ServiceArgumentModifiers {
	Required(ServiceDataTypes),
	Optional(ServiceDataTypes),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "desc")]
/// Describes a data type used by a service. The optional string should be a general type ID
/// identifying more specific information about the type. For example, the following could be
/// used to identify that a string uuid representing a fixture should be provided.
///
/// `ServiceDataTypes::String(Some(String::from("fixture-uid")))`
///
/// This can be used to provide auto-completion and inference in UI-driven configuration tools.
pub enum ServiceDataTypes {
	U8(Option<String>),
	U16(Option<String>),
	U32(Option<String>),
	I8(Option<String>),
	I16(Option<String>),
	I32(Option<String>),
	String(Option<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CallServiceError {
	TypeValidationFailed,
}

pub trait Service {

	/// Gets the ID of a service for use when calling it
	fn get_id<'a>(&'a self) -> &'a str;

	/// Gets the name of the service
	fn get_name<'a>(&'a self) -> &'a str;

	/// Gets a description indicating what the service does
	fn get_description<'a>(&'a self) -> &'a str;

	/// Get the documentation for the arguments required by the service
	fn get_signature<'a>(&'a self) -> (&'a [ServiceArgument], Option<'a ServiceArgument>);

	/// Call the service locally without static typing
	fn call(&self, arguments: Vec<Box<dyn Any>>) -> Result<Box<dyn Any>, CallServiceError>;

	/// Call the service using JSON values
	fn call_json(&self, arguments: Vec<Value>) -> Result<Value, CallServiceError>;
}

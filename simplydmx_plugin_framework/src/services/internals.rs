use std::{
	any::Any,
	pin::Pin,
	boxed::Box,
	future::Future,
};
use serde::{
	Serialize,
	Deserialize,
};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceArgument<'a> {
	pub id: &'a str,
	pub name: &'a str,
	pub description: &'a str,
	pub val_type: ServiceArgumentModifiers,

	/// Type ID identifying more specific information about the value. For example, the following could be
	/// used to identify that a string uuid representing a fixture should be provided.
	///
	/// `Some(String::from("fixture-uid"))`
	///
	/// This can be used to provide auto-completion and inference in UI-driven configuration tools.
	pub val_type_id: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "modifier", content = "value")]
pub enum ServiceArgumentModifiers {
	Required(ServiceDataTypes),
	Optional(ServiceDataTypes),
	Vector(ServiceDataTypes),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
/// Describes a data type used by a service.
pub enum ServiceDataTypes {
	U8,
	U16,
	U32,
	I8,
	I16,
	I32,
	String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CallServiceError {
	TypeValidationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CallServiceJSONError {
	DeserializationFailed,
	SerializationFailed,
}

pub trait Service {

	/// Gets the ID of a service for use when calling it
	fn get_id<'a>(&'a self) -> &'a str;

	/// Gets the name of the service
	fn get_name<'a>(&'a self) -> &'a str;

	/// Gets a description indicating what the service does
	fn get_description<'a>(&'a self) -> &'a str;

	/// Get the documentation for the arguments required by the service
	fn get_signature<'a>(&'a self) -> (&'a [ServiceArgument], &'a Option<ServiceArgument>);

	/// Call the service locally without static typing
	fn call<'a>(&'a self, arguments: Vec<Box<dyn Any + Sync + Send>>) -> Pin<Box<dyn Future<Output = Result<Box<dyn Any + Sync + Send>, CallServiceError>> + Send + 'a>>;

	/// Call the service using JSON values
	fn call_json<'a>(&'a self, arguments: Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, CallServiceJSONError>> + Send + 'a>>;
}

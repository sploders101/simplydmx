use std::{
	any::Any,
	pin::Pin,
	boxed::Box,
	future::Future,
};
use serde_json::Value;

use simplydmx_plugin_macros::portable;

#[portable]
pub struct ServiceArgument<'a> {
	pub id: &'a str,
	pub description: &'a str,
	pub val_type: &'a str,

	/// Type ID identifying more specific information about the value. For example, the following could be
	/// used to identify that a string uuid representing a fixture should be provided.
	///
	/// `Some(String::from("fixture-uid"))`
	///
	/// This can be used to provide auto-completion and inference in UI-driven configuration tools.
	pub val_type_hint: Option<&'a str>,
}

#[portable]
#[derive(Hash)]
pub struct ServiceArgumentOwned {
	pub id: String,
	pub description: String,
	pub val_type: String,
	pub val_type_hint: Option<String>,
}
impl From<ServiceArgument<'_>> for ServiceArgumentOwned {
	fn from(arg: ServiceArgument<'_>) -> Self {
		ServiceArgumentOwned {
			id: String::from(arg.id),
			description: String::from(arg.description),
			val_type: String::from(arg.val_type),
			val_type_hint: if let Some(hint) = arg.val_type_hint { Some(String::from(hint)) } else { None },
		}
	}
}

#[portable]
#[serde(tag = "type")]
pub enum CallServiceError {
	TypeValidationFailed,
}

#[portable]
#[serde(tag = "type")]
pub enum CallServiceRPCError {
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
	fn call_json<'a>(&'a self, arguments: Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, CallServiceRPCError>> + Send + 'a>>;

	// Call the service using CBOR values
	fn call_cbor<'a>(&'a self, arguments: Vec<Vec<u8>>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, CallServiceRPCError>> + Send + 'a>>;
}

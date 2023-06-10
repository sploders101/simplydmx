use std::{
	fmt,
	any::Any,
	marker::PhantomData,
};

use serde::{
	Serialize,
};

use serde_json::{
	self,
	Value,
};
use ciborium;

/// Marker trait that enables a message to be sent on the event bus.
///
/// This trait is automatically implemented when a type implements `Serialize` and `Deserialize`
/// from serde, and ensures that the infrastructure necessary to send and receive the type via
/// external communication methods is met.
///
/// This trait, and the traits leading up to it ensure that the message can be
/// sent over any binary stream transport.
pub trait BidirectionalPortable: PortableMessage + serde::de::DeserializeOwned { }
impl<T: PortableMessage + serde::de::DeserializeOwned> BidirectionalPortable for T { }

/// Common API for all events sent on the bus, regardless of type. Mainly
pub trait PortableMessage: Any + Sync + Send {
	fn serialize_json(&self) -> Result<Value, serde_json::Error>;
	fn serialize_cbor(&self) -> Result<Vec<u8>, String>;
	fn clone_portable_message(&self) -> Box<dyn PortableMessage>;
	fn fmt_portable_message(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

// Blanket message serializer implementation for all serializable data types
impl<T: Serialize + Clone + fmt::Debug + Sync + Send + 'static> PortableMessage for T {
	fn serialize_json(&self) -> Result<Value, serde_json::Error> {
		return serde_json::to_value(&self);
	}
	fn serialize_cbor(&self) -> Result<Vec<u8>, String> {
		let mut out = Vec::<u8>::new();
		match ciborium::ser::into_writer(self, &mut out) {
			Ok(()) => return Ok(out),
			Err(error) => match error {
				ciborium::ser::Error::Io(error) => return Err(format!("{:?}", error)),
				ciborium::ser::Error::Value(error) => return Err(error),
			},
		}
	}
	fn clone_portable_message(&self) -> Box<dyn PortableMessage> {
		return Box::new(Self::clone(self));
	}
	fn fmt_portable_message(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return fmt::Debug::fmt(&self, f);
    }
}

impl Clone for Box<dyn PortableMessage> {
	fn clone(&self) -> Self {
		return self.clone_portable_message();
	}
}

impl fmt::Debug for Box<dyn PortableMessage> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		return self.fmt_portable_message(f);
	}
}

/// Generic object used to create storable parsing functions
pub struct PortableMessageDeserializer<T: BidirectionalPortable>(PhantomData<T>);
impl<T: BidirectionalPortable> PortableMessageDeserializer<T> {
	pub fn new() -> Self {
		return PortableMessageDeserializer(PhantomData::<T>);
	}
}

/// Deserializer trait used to create a common interface for multiple deserialization functions
/// implemented with `PortableMessageDeserializer`
pub trait PortableMessageGenericDeserializer: Sync + Send + 'static {
	fn deserialize_json(&self, value: Value) -> Result<Box<dyn PortableMessage>, serde_json::Error>;
	fn deserialize_cbor(&self, value: &[u8]) -> Result<Box<dyn PortableMessage>, String>;
}

// Blanket deserializer implementation for all PortableMessageDeserializer instances
impl<T: BidirectionalPortable> PortableMessageGenericDeserializer for PortableMessageDeserializer<T> {
	fn deserialize_json(&self, value: Value) -> Result<Box<dyn PortableMessage>, serde_json::Error> {
		return match serde_json::from_value::<T>(value) {
			Ok(decoded) => { Ok(Box::new(decoded)) },
			Err(error) => { Err(error) },
		};
	}
	fn deserialize_cbor(&self, value: &[u8]) -> Result<Box<dyn PortableMessage>, String> {
		return match ciborium::de::from_reader::<T, &[u8]>(value) {
			Ok(decoded) => Ok(Box::new(decoded)),
			Err(error) => Err(format!("{:?}", error)),
		}
	}
}

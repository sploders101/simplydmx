use serde::de::DeserializeOwned;
use simplydmx_plugin_framework::*;

/// Data type used to hold a serialized instance of an arbitrary data type.
///
/// This is intended to encapsulate dynamically-typed data intended for deserialization by the output plugin
#[portable]
#[serde(tag = "t", content = "c")]
pub enum SerializedData {
	Bincode(Vec<u8>),
	JSON(serde_json::Value),
}
impl SerializedData {
	pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, DeserializeError> {
		let fixture_info: T = match self {
			SerializedData::Bincode(data) => bincode::deserialize(&data)?,
			SerializedData::JSON(data) => serde_json::from_value(data)?,
		};

		return Ok(fixture_info);
	}
}

/// Struct used to support `?` syntax for casting to `Err(...)`.
///
/// Returned when there is an error deserializing `SerializedData`
pub struct DeserializeError();
impl From<bincode::Error> for DeserializeError {
	fn from(_: bincode::Error) -> Self {
		return DeserializeError();
	}
}
impl From<serde_json::Error> for DeserializeError {
	fn from(_: serde_json::Error) -> Self {
		return DeserializeError();
	}
}

#[macro_export]
macro_rules! impl_deserialize_err {
	($type:ty, $output:expr) => {
		impl From<crate::utilities::serialized_data::DeserializeError> for $type {
			fn from(_: crate::utilities::serialized_data::DeserializeError) -> Self {
				return $output;
			}
		}
	};
}

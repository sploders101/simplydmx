use serde::de::DeserializeOwned;
use simplydmx_plugin_framework::*;

/// Data type used to hold a serialized instance of an arbitrary data type.
///
/// This is intended to encapsulate dynamically-typed data intended for deserialization by the output plugin
#[portable]
#[serde(untagged)]
pub enum SerializedData {
	Cbor(Vec<u8>),
	JSON(serde_json::Value),
}
impl SerializedData {
	pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, DeserializeError> {
		let fixture_info: T = match self {
			SerializedData::Cbor(data) => ciborium::de::from_reader::<'_, T, &[u8]>(&data)?,
			SerializedData::JSON(data) => serde_json::from_value(data)?,
		};

		return Ok(fixture_info);
	}
}

/// Struct used to support `?` syntax for casting to `Err(...)`.
///
/// Returned when there is an error deserializing `SerializedData`
pub struct DeserializeError();
impl<T> From<ciborium::de::Error<T>> for DeserializeError {
	fn from(_: ciborium::de::Error<T>) -> Self {
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

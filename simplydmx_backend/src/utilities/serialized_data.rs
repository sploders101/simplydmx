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
			SerializedData::Cbor(data) => ciborium::de::from_reader::<T, &[u8]>(&data)?,
			SerializedData::JSON(data) => serde_json::from_value(data)?,
		};

		return Ok(fixture_info);
	}
}

/// Struct used to support `?` syntax for casting to `Err(...)`.
///
/// Returned when there is an error deserializing `SerializedData`
pub struct DeserializeError(pub String);
impl<T> From<ciborium::de::Error<T>> for DeserializeError {
	fn from(err: ciborium::de::Error<T>) -> Self {
		return DeserializeError(match err {
			ciborium::de::Error::Syntax(_) => String::from("Syntax error"),
			ciborium::de::Error::Semantic(_, err) => err,
			_ => String::from("An unknown error occurred while deserializing"),
		});
	}
}
impl From<serde_json::Error> for DeserializeError {
	fn from(err: serde_json::Error) -> Self {
		return DeserializeError(err.to_string());
	}
}

#[macro_export]
macro_rules! impl_deserialize_err {
	($type:ty, $output:expr) => {
		impl From<crate::utilities::serialized_data::DeserializeError> for $type {
			fn from(err: crate::utilities::serialized_data::DeserializeError) -> Self {
				return $output(err.0);
			}
		}
	};
}

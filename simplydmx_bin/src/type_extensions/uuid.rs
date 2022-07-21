use std::{
	fmt,
	ops::Deref,
};

use uuid;
use serde::{
	Serializer,
	Deserializer,
	Serialize,
	Deserialize,
	de::{
		self,
		Visitor,
	},
};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct Uuid(uuid::Uuid);

impl Uuid {
	pub fn new() -> Self {
		return Uuid(uuid::Uuid::new_v4());
	}
}

impl Deref for Uuid {
	type Target = uuid::Uuid;

	fn deref(&self) -> &Self::Target {
		return &self.0;
	}
}

// ┌─────────────────┐
// │    Serialize    │
// └─────────────────┘

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
		if serializer.is_human_readable() {
			return serializer.serialize_str(&self.0.to_string());
		} else {
			return serializer.serialize_bytes(&self.0.to_bytes_le());
		}
    }
}

// ┌───────────────────┐
// │    Deserialize    │
// └───────────────────┘

struct UuidVisitor;
impl<'de> Visitor<'de> for UuidVisitor {
	type Value = Uuid;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		return formatter.write_str("a v4 UUID formatted as a string");
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		let uuid = uuid::Uuid::parse_str(value);
		if let Ok(uuid) = uuid {
			return Ok(Uuid(uuid));
		} else {
			return Err(E::custom("String is not a valid v4 UUID"));
		}
	}

	fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		let mut uuid_bytes = [0u8; 16];
		uuid_bytes.copy_from_slice(&value[0..16]);
		return Ok(Uuid(uuid::Uuid::from_bytes_le(uuid_bytes)));
	}
}

impl<'de> Deserialize<'de> for Uuid {
	fn deserialize<D>(deserializer: D) -> Result<Uuid, D::Error>
	where
		D: Deserializer<'de>,
	{
		if deserializer.is_human_readable() {
			return deserializer.deserialize_str(UuidVisitor);
		} else {
			return deserializer.deserialize_bytes(UuidVisitor);
		}
	}
}

// ┌───────────────┐
// │    Display    │
// └───────────────┘

impl fmt::Display for Uuid {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.to_string())?;
		return Ok(());
	}
}

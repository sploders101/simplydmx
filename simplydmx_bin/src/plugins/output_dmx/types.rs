use std::{
	collections::{
		HashMap,
	},
};
use async_std::{
	sync::{
		RwLock,
	},
};
use serde::{
	Serialize,
	Deserialize,
};

#[derive(Debug)]
pub struct OutputContext {
	pub output_types: RwLock<HashMap<String, OutputDescriptor>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputDescriptor {
	pub id: String,
	pub name: String,
	pub description: String,
	pub plugin_id: String,
	pub register_universe_id: String,
	pub delete_universe_id: String,
	pub output_channel: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayableOutput {
	pub id: String,
	pub name: String,
	pub description: String,
}

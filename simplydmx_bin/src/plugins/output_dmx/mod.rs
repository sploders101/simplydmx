pub mod services;
pub mod types;

use std::{
	sync::Arc,
	collections::HashMap,
};
use async_std::sync::Mutex;

use uuid::Uuid;
use simplydmx_plugin_framework::*;

pub async fn initialize(plugin_context: Arc<PluginContext>) {

}

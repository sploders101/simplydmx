mod services;
use services::GetBaseLayer;

use simplydmx_plugin_framework::*;

pub async fn initialize(plugin_context: PluginContext) {
	plugin_context.register_service(true, GetBaseLayer::new()).await.unwrap();
	plugin_context.declare_event::<()>(String::from("patcher.patch_updated")).await.unwrap();
}

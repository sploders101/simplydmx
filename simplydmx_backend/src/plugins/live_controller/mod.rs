pub mod types;
pub mod control_interfaces;
pub mod control_proxies;
pub mod scalable_value;
use simplydmx_plugin_framework::*;

pub struct ControllerInterface {
	plugin: PluginContext,
}
impl ControllerInterface {
	pub async fn init(plugin_framework: &PluginManager) -> anyhow::Result<Self> {
		let plugin = plugin_framework
			.register_plugin("live_controller", "Live Controller")
			.await
			.unwrap();

		return Ok(ControllerInterface {
			plugin,
		});
	}
}

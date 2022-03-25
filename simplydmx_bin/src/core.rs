use std::sync::Arc;

use simplydmx_plugin_framework::{
	PluginManager,
	PluginContext,
	interpolate_service,
};

#[interpolate_service(
	PluginManager,
	"shutdown",
	"Quit SimplyDMX",
	"Shut down SimplyDMX and all of its components",
)]
impl ShutdownService {
	#[service_main()]
	pub async fn shutdown(self) {
		self.0.shutdown().await;
	}
}

#[interpolate_service(
	PluginContext,
	"log",
	"Log",
	"Log a message somewhere useful",
)]
impl LogService {
	#[service_main(
		("Message", "The message to log"),
	)]
	pub async fn log(self, msg: String) {

		#[cfg(feature = "stdout-logging")]
		println!("{}", &msg);

		self.0.emit::<String>("log".into(), msg).await;
	}
}

pub async fn initialize(plugin_manager: PluginManager, plugin_context: PluginContext) {
	plugin_context.register_service(true, ShutdownService(Arc::new(plugin_manager.clone()))).await.unwrap();
	plugin_context.register_service(true, LogService(Arc::new(plugin_context.clone()))).await.unwrap();
}

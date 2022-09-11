use simplydmx_plugin_framework::*;

#[interpolate_service(
	"log",
	"Log",
	"Log a message somewhere useful",
)]
impl LogService {
	#![inner_raw(PluginContext)]
	#[service_main(
		("Message", "The message to log"),
	)]
	pub async fn log(self, msg: String) {

		#[cfg(feature = "stdout-logging")]
		println!("{}", &msg);

		self.0.emit::<String>("log".into(), FilterCriteria::None, msg).await;
	}
}

#[interpolate_service(
	"log_error",
	"Log Error",
	"Log an error that needs to be addressed",
)]
impl LogErrorService {
	#![inner_raw(PluginContext)]
	#[service_main(
		("Message", "The message to log"),
	)]
	pub async fn log(self, msg: String) {

		#[cfg(feature = "stdout-logging")]
		println!("{}", &msg);

		self.0.emit::<String>("log_error".into(), FilterCriteria::None, msg).await;
	}
}

pub async fn initialize(plugin_context: PluginContext) {
	plugin_context.register_service(true, LogService(plugin_context.clone())).await.unwrap();
	plugin_context.register_service(true, LogErrorService(plugin_context.clone())).await.unwrap();
}

// Mods
pub mod keep_alive;
pub mod event_emitter;
pub mod plugin;
pub mod services;

// Tests
#[cfg(test)]
mod tests;

// Re-exports
pub use services::internals::Service;

// Macros and macro re-exports
pub use simplydmx_plugin_macros::*;

#[macro_export]
macro_rules! service_docs {
	($id:literal, $name:literal, $description:literal) => {
		fn get_service_id_internal() -> &'static str {$id}
		fn get_service_name_internal() -> &'static str {$name}
		fn get_service_description_internal() -> &'static str {$description}
	};
}

#[macro_export]
macro_rules! call_service {
	($context:ident, $plugin:literal, $service:literal) => {
		$context.get_service($plugin, $service).await.unwrap().call(vec!()).await.unwrap();
	};
	($context:ident, $plugin:literal, $service:literal, $($args:expr),*) => {
		$context.get_service($plugin, $service).await.unwrap().call(vec!($(Box::new($args)),+)).await.unwrap();
	};
}

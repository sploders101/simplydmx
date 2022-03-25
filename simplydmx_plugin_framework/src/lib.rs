// Mods
mod keep_alive;
mod event_emitter;
mod plugin;
mod services;

// Tests
#[cfg(test)]
mod tests;

// Re-exports
pub use services::{
	internals::{
		Service,
		CallServiceError,
		CallServiceJSONError,
		ServiceArgument,
		ServiceArgumentModifiers,
		ServiceDataTypes,
	},
	type_specifiers::{
		TypeSpecifier,
		DropdownOptionNative,
		DropdownOptionJSON,
	}
};
pub use event_emitter::{
	EventEmitter,
	EventReceiver,
	ArcAny,
};
pub use keep_alive::{
	KeepAliveRegistrationError,
	KeepAliveDeregistrationError,
};
pub use plugin::{
	Dependency,
	GetServiceError,
	Plugin,
	PluginContext,
	PluginManager,
	PluginRegistry,
	RegisterPluginError,
	ServiceDescription,
	ServiceRegistrationError,
	TypeSpecifierRegistrationError,
	TypeSpecifierRetrievalError,
};

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

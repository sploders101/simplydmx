// Mods
mod keep_alive;
mod event_emitter;
mod plugin;
mod services;
mod arc_any;

// Tests
#[cfg(test)]
mod tests;

// Re-exports
pub use services::{
	internals::{
		Service,
		CallServiceError,
		CallServiceRPCError,
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
	FilterCriteria,
	ArcPortable,
	Event,
	PortableJSONEvent,
	PortableCborEvent,
	DeclareEventError,
	RegisterEncodedListenerError,
	RegisterListenerError,
	BidirectionalPortable,
	PortableMessage,
	PortableMessageDeserializer,
};
pub use keep_alive::{
	KeepAliveRegistrationError,
	KeepAliveDeregistrationError,
};
pub use arc_any::ArcAny;
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
macro_rules! call_service {
	($context:expr, $plugin:literal, $service:literal) => {
		$context.get_service($plugin, $service).await.unwrap().call(vec!()).await.unwrap();
	};
	($context:expr, $plugin:literal, $service:literal, $($args:expr),*) => {
		$context.get_service($plugin, $service).await.unwrap().call(vec!($(Box::new($args)),+)).await.unwrap();
	};
}

#[macro_export]
macro_rules! log_error {
	($context:expr, $($args:expr),*) => {
		call_service!($context, "core", "log_error", format!($($args),*));
	};
}

#[macro_export]
macro_rules! log {
	($context:expr, $($args:expr),*) => {
		call_service!($context, "core", "log", format!($($args),*));
	};
}

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

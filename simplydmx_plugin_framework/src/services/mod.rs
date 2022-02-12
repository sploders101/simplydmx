use std::collections::HashMap;

pub mod internals;
pub mod type_specifiers;

struct ServiceRegistry (HashMap<String, Box<dyn internals::Service>>);

impl ServiceRegistry {

}

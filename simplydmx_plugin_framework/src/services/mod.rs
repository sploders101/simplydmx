use std::collections::HashMap;

pub mod internals;

struct ServiceRegistry (HashMap<String, Box<dyn internals::Service>>);

impl ServiceRegistry {

}

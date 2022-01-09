use std::collections::HashMap;

use self::service::Service;

mod service;

struct ServiceRegistry (HashMap<String, Box<dyn Service>>);

impl ServiceRegistry {

}

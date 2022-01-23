#[macro_use]
extern crate simplydmx_plugin_framework;

use simplydmx_plugin_framework::Service;

#[derive(Service)]
struct TestService {
    id: &'static str,
    name: &'static str,
    description: &'static str,

}
impl TestService {
    pub fn new() -> TestService {
        return TestService {
            id: "test_service",
            name: "Test service",
            description: "Test service's description",
        };
    }

    #[interpolate_service]
    pub fn call_internal(&self, test1: u16, test2: u32) -> () {
    }
}

fn main() {
    let service = TestService::new();
    let service_description = service.get_signature();
    service.call(vec![]);
}

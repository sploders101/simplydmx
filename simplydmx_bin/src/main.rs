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

    #[interpolate_service(
        "This is the ID of the light that you would like to control",
        "This is the value you want to assign to the light (0-65535)",
    )]
    pub fn call_internal(&self, light_id: u16, value: u32) -> () {
        // Do stuff here
    }
}

fn main() {
    let service = TestService::new();
    let service_description = service.get_signature();
    service.call(vec![]);
}

use simplydmx_plugin_framework::*;

#[portable]
pub struct FormDescriptor();
impl FormDescriptor {
	pub fn new() -> FormDescriptor {
		return FormDescriptor();
	}
}

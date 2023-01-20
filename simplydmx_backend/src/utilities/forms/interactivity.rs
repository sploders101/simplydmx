use simplydmx_plugin_framework::*;

#[portable]
/// Describes a function that can be used to add interactivity to a form
pub enum InteractiveDescription {
	Not(Box<InteractiveDescription>),
	And(Vec<InteractiveDescription>),
	Or(Vec<InteractiveDescription>),
	Equal {
		field_name: String,
		value: serde_json::Value,
	},
}

impl InteractiveDescription {
	pub fn not(desc: InteractiveDescription) -> Self {
		return InteractiveDescription::Not(Box::new(desc));
	}
}

use simplydmx_plugin_framework::*;


#[portable]
#[derive(Default)]
/// Describes validation criteria for a number input
pub enum NumberValidation {
	#[default]
	None,
	And(Vec<NumberValidation>),
	Or(Vec<NumberValidation>),
	Between(i32, i32),
	DivisibleBy(i32),
}

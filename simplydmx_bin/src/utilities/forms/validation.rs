use simplydmx_plugin_framework::*;


#[portable]
/// Describes validation criteria for a number input
pub enum NumberValidation {
	None,
	Not(Box<NumberValidation>),
	And(Vec<NumberValidation>),
	Or(Vec<NumberValidation>),
	Between(f64, f64),
	DivisibleBy(f64),
}

pub trait ImplicitNumberValidation {
	fn validate_type_only() -> NumberValidation;

	fn with_validation(validation: NumberValidation) -> NumberValidation {
		return NumberValidation::And(vec![Self::validate_type_only(), validation]);
	}
}

macro_rules! validate {
	($type:ident, $base:expr) => {
		impl ImplicitNumberValidation for $type {
			fn validate_type_only() -> NumberValidation { $base }
		}
	}
}

validate!(u8, NumberValidation::And(vec![
	NumberValidation::Between(0.0, 255.0),
	NumberValidation::DivisibleBy(1.0),
]));

validate!(u16, NumberValidation::And(vec![
	NumberValidation::Between(0.0, 65535.0),
	NumberValidation::DivisibleBy(1.0),
]));

validate!(u32, NumberValidation::And(vec![
	NumberValidation::Between(0.0, 4294967295.0),
	NumberValidation::DivisibleBy(1.0),
]));

validate!(i8, NumberValidation::And(vec![
	NumberValidation::Between(-128.0, 127.0),
	NumberValidation::DivisibleBy(1.0),
]));

validate!(i16, NumberValidation::And(vec![
	NumberValidation::Between(-32_768.0, 32_767.0),
	NumberValidation::DivisibleBy(1.0),
]));

validate!(i32, NumberValidation::And(vec![
	NumberValidation::Between(-2_147_483_648.0, 2_147_483_647.0),
	NumberValidation::DivisibleBy(1.0),
]));

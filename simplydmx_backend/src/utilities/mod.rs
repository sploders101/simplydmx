pub mod forms;
pub mod serialized_data;

#[macro_export]
macro_rules! impl_anyhow {
	($type:ident, $variant:expr) => {
		impl From<anyhow::Error> for $type {
			fn from(value: anyhow::Error) -> Self {
				return $variant(format!("{:?}", value));
			}
		}
	}
}

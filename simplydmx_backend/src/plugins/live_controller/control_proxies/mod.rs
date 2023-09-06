//! This module contains proxies designed to assist in the
//! development of controller integrations.

mod debounce_analog;
pub use debounce_analog::DebounceAnalog;
mod analog_to_digital;
pub use analog_to_digital::AnalogToBoolean;
mod digital_to_analog;
pub use digital_to_analog::BooleanToAnalog;

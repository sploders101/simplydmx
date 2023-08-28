//! This file defines trait interfaces for interacting with physical controls of different types

use async_trait::async_trait;

use super::scalable_value::ScalableValue;

pub type Action<T> = Box<dyn Fn(T) -> () + Send + Sync + 'static>;

#[async_trait]
pub trait BooleanInterface {
	/// Sets the action to take when an input is received
	fn set_bool_action(&self, action: Option<Action<bool>>);
	/// Sends feedback through the interface. Returns true if successful, false if not.
	async fn send_bool(&self, _state: bool) -> bool { false }
	/// Sets the action to take when an input is received. Velocity may be `None` to indicate
	/// the device does not support it.
	fn set_bool_with_velocity_action(&self, action: Option<Action<(bool, Option<ScalableValue>)>>);
	/// Sends feedback through the interface. Returns true if successful, false if not.
	async fn send_bool_with_velocity(&self, _state: (bool, ScalableValue)) -> bool { false }
}

#[async_trait]
pub trait AnalogInterface {
	/// Sets the action to take when an input is received
	fn set_analog_action(&self, action: Option<Action<ScalableValue>>);
	/// Sends feedback through the interface. Returns true if successful, false if not.
	async fn send_analog(&self, _value: ScalableValue) -> bool { false }
}

pub enum TickingEncoderMovement {
	Up(u8),
	Down(u8),
}

#[async_trait]
/// Represents an interface that can move infinitely in one dimension, representing movement
/// deltas represented as "ticks". One tick is one click of the wheel, and corresponds to an
/// increase/decrease of 1 on the value it controls.
pub trait TickingEncoderInterface {
	/// Sets the action to take when an input is received
	fn set_ticking_encoder_action(&self, action: Option<Action<TickingEncoderMovement>>);
	/// Sends feedback through the interface. Returns true if successful, false if not.
	async fn send_ticking_encoder(&self, _value: TickingEncoderMovement) -> bool { false }
}

pub enum PreciseEncoderMovement {
	Up(f64),
	Down(f64),
}

#[async_trait]
/// Represents an interface that can move infinitely in one dimension, representing movement
/// deltas represented as "ticks". One tick is approximately one degree of rotation.
pub trait EncoderInterface {
	/// Sets the action to take when an input is received
	fn set_encoder_action(&self, action: Option<Action<PreciseEncoderMovement>>);
	/// Sends feedback through the interface. Returns true if successful, false if not.
	async fn send_encoder(&self, _value: PreciseEncoderMovement) -> bool { false }
}

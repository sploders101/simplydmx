//! This file defines trait interfaces for interacting with physical controls of different types

use async_trait::async_trait;

use super::scalable_value::ScalableValue;

#[async_trait]
pub trait BooleanInterface {
	/// Receives a new input from the interface.
	/// This waits indefinitely, and a `None` value indicates the source no longer exists.
	async fn recv_bool(&self) -> Option<bool>;
	/// Sends feedback through the interface. Returns true if successful, false if not.
	async fn send_bool(&self, state: bool) -> bool { false }
}

#[async_trait]
pub trait BooleanInterfaceWithVelocity {
	/// Receives a new input from the interface.
	/// This waits indefinitely, and a `None` value indicates the source no longer exists.
	async fn recv_bool_with_velocity(&self) -> Option<(bool, ScalableValue)>;
	/// Sends feedback through the interface. Returns true if successful, false if not.
	async fn send_bool_with_velocity(&self, state: (bool, ScalableValue)) -> bool { false }
}

#[async_trait]
pub trait AnalogInterface {
	/// Receives a new input from the interface.
	/// This waits indefinitely, and a `None` value indicates the source no longer exists.
	async fn recv_analog(&self) -> Option<ScalableValue>;
	/// Sends feedback through the interface. Returns true if successful, false if not.
	async fn send_analog(&self, value: ScalableValue) -> bool { false }
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
	/// Receives a new input from the interface.
	/// This waits indefinitely, and a `None` value indicates the source no longer exists.
	async fn recv_ticking_encoder(&self) -> Option<TickingEncoderMovement>;
	/// Sends feedback through the interface. Returns true if successful, false if not.
	async fn send_ticking_encoder(&self, value: TickingEncoderMovement) -> bool { false }
}

pub enum PreciseEncoderMovement {
	Up(f64),
	Down(f64),
}

#[async_trait]
/// Represents an interface that can move infinitely in one dimension, representing movement
/// deltas represented as "ticks". One tick is approximately one degree of rotation.
pub trait EncoderInterface {
	/// Receives a new input from the interface.
	/// This waits indefinitely, and a `None` value indicates the source no longer exists.
	async fn recv_encoder(&self) -> Option<PreciseEncoderMovement>;
	/// Sends feedback through the interface. Returns true if successful, false if not.
	async fn send_encoder(&self, value: PreciseEncoderMovement) -> bool { false }
}

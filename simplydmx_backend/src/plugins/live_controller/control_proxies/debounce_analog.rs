//! This file defines proxy types used to add extra functionality
//! to physical control interfaces by polyfilling.

use std::cmp::Ordering;
use async_trait::async_trait;
use tokio::sync::Mutex;

use super::{control_interfaces::AnalogInterface, scalable_value::ScalableValue};


/// Debounces analog I/O so the physical input must cross the
/// logical state value before taking effect.
///
/// This control modifier is useful, for example, when using faders
/// that aren't motorized. If the user pushes the fader to 100%, and
/// then the submaster is set to 50% in software, the user must move
/// the fader to 50% before any further movement takes effect.
pub struct DebounceAnalog {
	state: Mutex<DebounceAnalogState>,
	inner: Box<dyn AnalogInterface + Send + Sync + 'static>,
}
enum DebounceAnalogLock {
	Unlocked,
	/// Locked until the physical value is >= the logical value
	LockedGte,
	/// Locked until the physical value is <= the logical value
	LockedLte,
}
struct DebounceAnalogState {
	locked: DebounceAnalogLock,
	logical_state: ScalableValue,
	physical_state: ScalableValue,
}
impl DebounceAnalog {
	/// Wraps a control interface in a `DebounceAnalog` interface
	pub fn wrap(inner: Box<dyn AnalogInterface + Send + Sync + 'static>) -> Self {
		return Self {
			state: Mutex::new(DebounceAnalogState {
				locked: DebounceAnalogLock::Unlocked,
				logical_state: ScalableValue::U8(0),
				physical_state: ScalableValue::U8(0),
			}),
			inner,
		};
	}
}
#[async_trait]
impl AnalogInterface for DebounceAnalog {
	async fn recv_analog(&self) -> Option<ScalableValue> {
		loop {
			let result = self.inner.recv_analog().await;
			match result {
				Some(result) => {
					let mut state = self.state.lock().await;
					state.physical_state = result;
					match state.locked {
						DebounceAnalogLock::Unlocked => {
							return Some(result);
						}
						DebounceAnalogLock::LockedGte => {
							if result >= state.logical_state {
								state.locked = DebounceAnalogLock::Unlocked;
								return Some(result);
							}
						}
						DebounceAnalogLock::LockedLte => {
							if result <= state.logical_state {
								state.locked = DebounceAnalogLock::Unlocked;
								return Some(result);
							}
						}
					}
				}
				None => {
					return None;
				}
			}
		}
	}
	async fn send_analog(&self, value: ScalableValue) -> bool {
		let mut state = self.state.lock().await;
		state.logical_state = value;
		match value.cmp(&state.physical_state) {
			Ordering::Less => state.locked = DebounceAnalogLock::LockedLte,
			Ordering::Equal => state.locked = DebounceAnalogLock::Unlocked,
			Ordering::Greater => state.locked = DebounceAnalogLock::LockedGte,
		}
		return true;
	}
}

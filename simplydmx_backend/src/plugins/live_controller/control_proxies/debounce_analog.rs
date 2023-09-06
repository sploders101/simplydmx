//! This file defines proxy types used to add extra functionality
//! to physical control interfaces by polyfilling.

use async_trait::async_trait;
use std::sync::Mutex;
use std::{cmp::Ordering, sync::Arc};

use super::super::{
	control_interfaces::{Action, AnalogInterface},
	scalable_value::ScalableValue,
};

/// Debounces analog I/O so the physical input must cross the
/// logical state value before taking effect.
///
/// This control modifier is useful, for example, when using faders
/// that aren't motorized. If the user pushes the fader to 100%, and
/// then the submaster is set to 50% in software, the user must move
/// the fader to 50% before any further movement takes effect.
pub struct DebounceAnalog {
	state: Arc<Mutex<DebounceAnalogState>>,
	wraps: Arc<dyn AnalogInterface + Send + Sync + 'static>,
}
enum DebounceAnalogLock {
	Unlocked,
	/// Locked until the physical value is >= the logical value
	LockedGte,
	/// Locked until the physical value is <= the logical value
	LockedLte,
}
struct DebounceAnalogState {
	action: Option<Action<ScalableValue>>,
	locked: DebounceAnalogLock,
	logical_state: ScalableValue,
	physical_state: ScalableValue,
}
impl DebounceAnalog {
	/// Wraps a control interface in a `DebounceAnalog` interface
	pub async fn new(inner: Arc<dyn AnalogInterface + Send + Sync + 'static>) -> Self {
		let state = Arc::new(Mutex::new(DebounceAnalogState {
			action: None,
			locked: DebounceAnalogLock::Unlocked,
			logical_state: ScalableValue::U8(0),
			physical_state: ScalableValue::U8(0),
		}));
		let state_ref = Arc::clone(&state);
		inner.set_analog_action(Some(Box::new(move |new_value: ScalableValue| {
			let mut state = state_ref.lock().unwrap();
			// Re-implement recv_analog
			let result = new_value;
			state.physical_state = result;
			match state.locked {
				DebounceAnalogLock::Unlocked => {
					state.action.as_ref().map(|action| action(result));
				}
				DebounceAnalogLock::LockedGte => {
					if result >= state.logical_state {
						state.locked = DebounceAnalogLock::Unlocked;
						state.action.as_ref().map(|action| action(result));
					}
				}
				DebounceAnalogLock::LockedLte => {
					if result <= state.logical_state {
						state.locked = DebounceAnalogLock::Unlocked;
						state.action.as_ref().map(|action| action(result));
					}
				}
			}
		}))).await;
		return DebounceAnalog {
			state,
			wraps: inner,
		};
	}
}
#[async_trait]
impl AnalogInterface for DebounceAnalog {
	async fn set_analog_action(&self, action: Option<Action<ScalableValue>>) {
		self.state.lock().unwrap().action = action;
	}
	async fn send_analog(&self, value: ScalableValue) -> bool {
		let mut state = self.state.lock().unwrap();
		state.logical_state = value;
		match value.cmp(&state.physical_state) {
			Ordering::Less => state.locked = DebounceAnalogLock::LockedLte,
			Ordering::Equal => state.locked = DebounceAnalogLock::Unlocked,
			Ordering::Greater => state.locked = DebounceAnalogLock::LockedGte,
		}
		return true;
	}
}

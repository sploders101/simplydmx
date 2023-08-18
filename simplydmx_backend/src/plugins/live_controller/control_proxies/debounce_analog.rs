//! This file defines proxy types used to add extra functionality
//! to physical control interfaces by polyfilling.

use std::{cmp::Ordering, sync::Arc};
use async_trait::async_trait;
use std::sync::Mutex;

use super::super::{control_interfaces::{Action, AnalogInterface}, scalable_value::ScalableValue};


/// Debounces analog I/O so the physical input must cross the
/// logical state value before taking effect.
///
/// This control modifier is useful, for example, when using faders
/// that aren't motorized. If the user pushes the fader to 100%, and
/// then the submaster is set to 50% in software, the user must move
/// the fader to 50% before any further movement takes effect.
#[derive(Clone)]
pub struct DebounceAnalog (Arc<DebounceAnalogInner>);
pub struct DebounceAnalogInner {
	state: Mutex<DebounceAnalogState>,
	action: Option<Action<ScalableValue>>,
	/// Keeps ownership of the original object when permanently wrapping
	wraps: Option<Box<dyn AnalogInterface + Send + Sync + 'static>>,
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
	fn handle_value(&self, new_value: ScalableValue) {
		if let Some(action) = self.0.action {
			// Re-implement recv_analog
			let result = new_value;
			let mut state = self.0.state.lock().unwrap();
			state.physical_state = result;
			match state.locked {
				DebounceAnalogLock::Unlocked => {
					return action(result);
				}
				DebounceAnalogLock::LockedGte => {
					if result >= state.logical_state {
						state.locked = DebounceAnalogLock::Unlocked;
						return action(result);
					}
				}
				DebounceAnalogLock::LockedLte => {
					if result <= state.logical_state {
						state.locked = DebounceAnalogLock::Unlocked;
						return action(result);
					}
				}
			}
		}
	}
	/// Registers a new DebounceAnalog interface as an intermediary action
	pub fn register(interface: &dyn AnalogInterface) -> Self {
		let new_debounce = Self(Arc::new(DebounceAnalogInner {
			action: None,
			state: Mutex::new(DebounceAnalogState {
				locked: DebounceAnalogLock::Unlocked,
				logical_state: ScalableValue::U8(0),
				physical_state: ScalableValue::U8(0),
			}),
			wraps: None,
		}));
		let self_ref = new_debounce.clone();
		interface.set_analog_action(Some(Box::new(move |new_value: ScalableValue| {
			self_ref.handle_value(new_value);
		})));
		return new_debounce;
	}
	/// Wraps a control interface in a `DebounceAnalog` interface
	pub fn wrap(mut inner: Box<dyn AnalogInterface + Send + Sync + 'static>) -> Self {
		let new_debounce = Self(Arc::new(DebounceAnalogInner {
			action: None,
			state: Mutex::new(DebounceAnalogState {
				locked: DebounceAnalogLock::Unlocked,
				logical_state: ScalableValue::U8(0),
				physical_state: ScalableValue::U8(0),
			}),
			wraps: Some(inner),
		}));
		let self_ref = new_debounce.clone();
		inner.set_analog_action(Some(Box::new(move |new_value: ScalableValue| {
			self_ref.handle_value(new_value);
		})));
		return new_debounce;
	}
}
#[async_trait]
impl AnalogInterface for DebounceAnalog {
	fn set_analog_action(&mut self, action: Option<Action<ScalableValue>>) {
		self.0.action = action;
	}
	async fn send_analog(&self, value: ScalableValue) -> bool {
		let mut state = self.0.state.lock().unwrap();
		state.logical_state = value;
		match value.cmp(&state.physical_state) {
			Ordering::Less => state.locked = DebounceAnalogLock::LockedLte,
			Ordering::Equal => state.locked = DebounceAnalogLock::Unlocked,
			Ordering::Greater => state.locked = DebounceAnalogLock::LockedGte,
		}
		return true;
	}
}

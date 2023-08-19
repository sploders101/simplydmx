//! This file defines proxy types used to add extra functionality
//! to physical control interfaces by polyfilling.

use std::sync::Arc;
use async_trait::async_trait;
use std::sync::Mutex;

use super::super::{scalable_value::ScalableValue, control_interfaces::{Action, AnalogInterface, BooleanInterface}};


pub struct AnalogToBoolean {
	state: Arc<Mutex<InnerState>>,
	wraps: Arc<dyn AnalogInterface + Send + Sync + 'static>,
}
struct InnerState {
	action: Option<Action<bool>>,
	last_value: Option<bool>,
}
impl AnalogToBoolean {
	pub fn new(wraps: Arc<dyn AnalogInterface + Send + Sync + 'static>, threshold: ScalableValue) -> Self {
		let state = Arc::new(Mutex::new(InnerState {
			action: None,
			last_value: None,
		}));
		let state_ref = Arc::clone(&state);
		wraps.set_analog_action(Some(Box::new(|value: ScalableValue| {
			let new_state = value >= threshold;
			let mut inner_state = state_ref.lock().unwrap();
			match inner_state.last_value {
				Some(state) if state != new_state => {
					inner_state.action.map(|action| action(new_state));
				}
				None => {
					inner_state.action.map(|action| action(new_state));
				}
				_ => {}
			}
			inner_state.last_value = Some(new_state);
		})));
		return Self {
			state,
			wraps,
		}
	}
}

#[async_trait]
impl BooleanInterface for AnalogToBoolean {
	fn set_bool_action(&self, action: Option<Action<bool>>) {
		self.state.lock().unwrap().action = action;
	}
	async fn send_bool(&self, state: bool) -> bool {
		if self.wraps.send_analog(if state { ScalableValue::U8(255) } else { ScalableValue::U8(0) }).await {
			self.state.lock().unwrap().last_value = Some(state);
			return true;
		}
		return false;
	}
	fn set_bool_with_velocity_action(&self, action: Option<Action<(bool, Option<ScalableValue>)>>) {
		self.state.lock().unwrap().action = match action {
			Some(action) => Some(Box::new(move |input: bool| action((input, None)))),
			None => None,
		};
	}
	async fn send_bool_with_velocity(&self, state: (bool, ScalableValue)) -> bool {
		if self.wraps.send_analog(if state.0 { ScalableValue::U8(255) } else { ScalableValue::U8(0) }).await {
			self.state.lock().unwrap().last_value = Some(state.0);
			return true;
		}
		return false;
	}
}

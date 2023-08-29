//! This file defines proxy types used to add extra functionality
//! to physical control interfaces by polyfilling.

use async_trait::async_trait;
use std::sync::Arc;
use std::sync::Mutex;

use super::super::{
	control_interfaces::{Action, AnalogInterface, BooleanInterface},
	scalable_value::ScalableValue,
};


pub struct BooleanToAnalog {
	state: Arc<Mutex<InnerState>>,
	wraps: Arc<dyn BooleanInterface + Send + Sync + 'static>,
}
struct InnerState {
	action: Option<Action<ScalableValue>>,
	last_value: Option<bool>,
}
impl BooleanToAnalog {
	pub fn new(wraps: Arc<dyn BooleanInterface + Send + Sync + 'static>) -> Self {
		let state = Arc::new(Mutex::new(InnerState {
			action: None,
			last_value: None,
		}));
		let state_ref = Arc::clone(&state);
		wraps.set_bool_action(Some(Box::new(move |value: bool| {
			let mut state_ref = state_ref.lock().unwrap();
			if Some(value) != state_ref.last_value {
				state_ref.last_value = Some(value);
				if let Some(ref mut action) = state_ref.action {
					action(if value { ScalableValue::U8(255) } else { ScalableValue::U8(0) });
				}
			}
		})));
		return Self {
			state,
			wraps,
		}
	}
}

#[async_trait]
impl AnalogInterface for BooleanToAnalog {
	fn set_analog_action(&self, action: Option<Action<ScalableValue>>) {
		self.state.lock().unwrap().action = action;
	}
	async fn send_analog(&self, value: ScalableValue) -> bool {
		return self.wraps.send_bool(value > ScalableValue::U8(127)).await;
	}
}

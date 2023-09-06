use std::sync::Arc;

use rustc_hash::FxHashMap;
use uuid::Uuid;

use super::{
	control_interfaces::{AnalogInterface, BooleanInterface},
	controller_services::ControllerServiceLink,
};

#[derive(Clone)]
/// Describes a controller
pub struct Controller {
	/// Contains a mapping of `control_id` to the control's state object
	controls: FxHashMap<Uuid, ControlState>,
}
impl Controller {
	pub fn new() -> Controller {
		return Self {
			controls: FxHashMap::default(),
		};
	}
}

#[derive(Clone)]
/// Holds all of a control's state
pub struct ControlState {
	/// Holds an interface to the service that is currently bound to the control
	binding: Option<Arc<dyn ControllerServiceLink + Send + Sync + 'static>>,
	/// Holds the control itself
	control: Control,
}
impl ControlState {
	pub fn new(control: Control) -> Self {
		return Self {
			binding: None,
			control,
		};
	}
}

#[derive(Clone)]
/// Describes the type and capabilities of a control.
pub enum Control {
	FaderColumn(ControlInstance<FaderColumnControl>),
	Fader(ControlInstance<FaderControl>),
	Knob(ControlInstance<KnobControl>),
	Button(ControlInstance<ButtonControl>),
}
impl From<ControlInstance<FaderColumnControl>> for Control {
	fn from(value: ControlInstance<FaderColumnControl>) -> Self {
		return Control::FaderColumn(value);
	}
}
impl From<ControlInstance<FaderControl>> for Control {
	fn from(value: ControlInstance<FaderControl>) -> Self {
		return Control::Fader(value);
	}
}
impl From<ControlInstance<KnobControl>> for Control {
	fn from(value: ControlInstance<KnobControl>) -> Self {
		return Control::Knob(value);
	}
}
impl From<ControlInstance<ButtonControl>> for Control {
	fn from(value: ControlInstance<ButtonControl>) -> Self {
		return Control::Button(value);
	}
}

#[derive(Clone)]
/// Describes a single distinct control on the board
pub struct ControlInstance<T> {
	/// The name of the control in the configuration screen
	name: Arc<str>,
	/// The control's capabilities. Controls may have multiple
	/// ways of interacting with them (despite being a single
	/// control), or extra metadata.
	capabilities: T,
}
impl<T> ControlInstance<T> {
	pub fn new(name: Arc<str>, control: T) -> Self {
		return Self {
			name: Arc::from(name),
			capabilities: control,
		};
	}
}

#[derive(Clone)]
/// Describes a group of controls tightly coupled with a fader
pub struct FaderColumnControl {
	fader: FaderControl,
	flash_btn: Option<ButtonControl>,
}
impl FaderColumnControl {
	pub fn build(fader: FaderControl) -> Self {
		return FaderColumnControl {
			fader,
			flash_btn: None,
		};
	}
	pub fn add_flash_btn(&mut self, flash_btn: ButtonControl) {
		self.flash_btn = Some(flash_btn);
	}
}

#[derive(Clone)]
/// Describes a single fader control
pub struct FaderControl {
	position: Arc<dyn AnalogInterface + Send + Sync + 'static>,
	touch: Option<Arc<dyn BooleanInterface + Send + Sync + 'static>>,
}
impl FaderControl {
	pub fn build(position: Arc<dyn AnalogInterface + Send + Sync + 'static>) -> Self {
		return Self {
			position,
			touch: None,
		};
	}
	pub fn add_touch(&mut self, touch: Arc<dyn BooleanInterface + Send + Sync + 'static>) {
		self.touch = Some(touch);
	}
}

#[derive(Clone)]
/// Describes a rotary knob input
pub struct KnobControl {
	/// Indicates whether the knob can communicate position
	position: Arc<dyn AnalogInterface + Send + Sync + 'static>,
	/// Indicates whether the knob can be pushed like a button
	push: Option<Arc<dyn BooleanInterface + Send + Sync + 'static>>,
}
impl KnobControl {
	pub fn build(position: Arc<dyn AnalogInterface + Send + Sync + 'static>) -> Self {
		return Self {
			position,
			push: None,
		};
	}
	pub fn add_push(&mut self, push: Arc<dyn BooleanInterface + Send + Sync + 'static>) {
		self.push = Some(push);
	}
}

#[derive(Clone)]
/// Describes a button input
pub struct ButtonControl {
	/// Indicates whether the button can communicate push events
	push: Arc<dyn BooleanInterface + Send + Sync + 'static>,
	/// Indicates whether the button can receive velocity events
	velocity: bool,
}
impl ButtonControl {
	pub fn build(push: Arc<dyn BooleanInterface + Send + Sync + 'static>) -> Self {
		return Self {
			push,
			velocity: false,
		};
	}
	pub fn enable_velocity(&mut self) {
		self.velocity = true;
	}
}

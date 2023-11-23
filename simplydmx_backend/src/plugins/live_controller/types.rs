use rustc_hash::FxHashMap;
use uuid::Uuid;

use super::controller_services::ControllerServiceLink;

/// Describes a controller
pub struct Controller {
	/// Contains a mapping of `control_id` to the control's state object
	pub controls: FxHashMap<Uuid, Control>,
}
impl Controller {
	pub fn new() -> Controller {
		return Self {
			controls: FxHashMap::default(),
		};
	}
}

/// Holds all of a control's state
pub struct Control {
	/// Holds an interface to the service that is currently bound to the control
	binding: Option<Box<dyn ControllerServiceLink + Send + Sync + 'static>>,
}
impl Control {
	pub fn new() -> Self {
		return Self {
			binding: None,
		};
	}
	pub async fn unbind(&mut self) {
	}
}

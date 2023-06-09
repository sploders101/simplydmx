use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;
use uuid::Uuid;


#[portable]
pub struct LiveControlState {
	controller_profiles: FxHashMap<Uuid, ControllerProfile>,
	controller_instances: FxHashMap<Uuid, ControllerInstance>,
}

#[portable]
/// Describes a particular instance of a controller
pub struct ControllerInstance {
	id: Uuid,
	name: String,
}

#[portable]
/// Defines the structure of a controller.
///
/// This can be referenced by multiple instances
pub struct ControllerProfile {
	name: String,
	control_groups: FxHashMap<Uuid, ControlGroup>,
}

#[portable]
/// Represents a group of distinct controls that are intended
/// to be used together
pub struct ControlGroup {
	name: String,
	/// The primary control within the group. This is usually
	/// the largest or most-often used.
	primary: ControlInstance,
	/// The flash button is usually the button right below a
	/// fader. It is normally used as a momentary override to
	/// set the submaster to 100%, returning to the fader's
	/// position upon release.
	flash_btn: Option<ControlInstance>,
}

#[portable]
/// Describes a single distinct control on the board
pub struct ControlInstance {
	/// The name of the control. This should always be presented
	/// within the context of the containing group, so it does
	/// not need to make sense on its own.
	name: Option<String>,
	/// The control's capabilities. Controls may have multiple
	/// ways of interacting with them (despite being a single
	/// control), or extra metadata.
	capabilities: ControlCapabilities,
}

#[portable]
/// Describes the type and capabilities of a control.
pub enum ControlCapabilities {
	Fader(FaderCapabilities),
	Knob(KnobCapabilities),
	Button(ButtonCapabilities),
}

#[portable]
/// Describes a single fader control
pub struct FaderCapabilities {
	position: bool,
	position_feedback: bool,
	touch: bool,
}

#[portable]
/// Describes a rotary knob input
pub struct KnobCapabilities {
	/// Indicates whether the knob can communicate position
	position: bool,
	/// Indicates whether the knob can receive position updates
	position_feedback: bool,
	/// Indicates whether the knob can be pushed like a button
	push: bool,
	/// Indicates whether the knob can receive button push updates
	/// (ie. for toggles)
	push_feedback: bool,
}

#[portable]
/// Describes a button input
pub struct ButtonCapabilities {
	/// Indicates whether the button can communicate push events
	push: bool,
	/// Indicates whether the button can receive push events
	push_feedback: bool,
	/// Indicates whether the button can receive velocity events
	velocity: bool,
}

//! This module provides services that can be linked to control surfaces

use std::{borrow::Cow, sync::Arc};

use crate::utilities::{forms::FormDescriptor, serialized_data::SerializedData};
use async_trait::async_trait;
use simplydmx_plugin_framework::*;
use thiserror::Error;
use tokio::sync::mpsc::Sender;

use super::scalable_value::ScalableValue;


#[portable]
#[derive(Debug, Clone, Error)]
pub enum ControllerLinkError {
	#[error("Bad form data.")]
	BadFormData,
	#[error("Bad form data. Details: {0}")]
	BadFormDataWithDetails(Cow<'static, str>),
	#[error("Error: {0}")]
	Other(Cow<'static, str>),
}

#[portable]
#[derive(Debug, Clone, Error)]
pub enum ControllerRestoreError {
	#[error("Bad form data.")]
	BadSaveData,
	#[error("Bad form data. Details: {0}")]
	BadSaveDataWithDetails(Cow<'static, str>),
	#[error("Error: {0}")]
	Other(Cow<'static, str>),
}

#[async_trait]
pub trait ControllerService {
	/// Gets a form for integrating a control group with a service
	///
	/// If prefilled is provided, the form data should be pre-filled with
	/// the data from `prefilled`. This data originates from the controller
	/// service link during editing.
	///
	/// `control_group` is an `Arc` containing a reference to the control
	/// group that is being edited.
	async fn get_form(
		&self,
		prefilled: Option<SerializedData>,
	) -> FormDescriptor;

	/// Links a control to the service
	async fn create_link(
		&self,
		from_save: bool,
		form_data: SerializedData,
		capabilities: ControlCapabilities,
	) -> Result<Box<dyn ControllerServiceLink + Send + Sync + 'static>, ControllerLinkError>;
}

/// Used for signaling updates to a control.
///
/// This can contain either a differential or full update for the control,
/// with logic used to determine the values for multiple display methods.
pub trait ControlUpdate {
	/// Gets the label for the control, to be displayed by the controller
	fn get_label(&self) -> Option<Arc<str>> { None }
	/// Gets a textual representation of the value
	fn get_value_text(&self) -> Option<Arc<str>> { None }
	/// Gets the value to be used with a motorized fader
	fn get_fader(&self) -> Option<ScalableValue> { None }
	/// Gets the value to be displayed on a rotary encoder or motorized knob
	fn get_knob(&self) -> Option<ScalableValue> { self.get_fader() }
	/// Gets the boolean value to be displayed by an LED
	fn get_led(&self) -> Option<bool> { None }
	/// Gets a variable brightness to be displayed by an LED
	fn get_led_brightness(&self) -> Option<ScalableValue> {
		if self.get_led()? {
			return Some(ScalableValue::U16(u16::MAX));
		} else {
			return Some(ScalableValue::U16(u16::MIN));
		}
	}
	/// Gets an RGB color value to be displayed by an LED
	fn get_rgb_led(&self) -> Option<(ScalableValue, ScalableValue, ScalableValue)> {
		let brightness = self.get_led_brightness()?;
		return Some((brightness, brightness, brightness));
	}
}

#[async_trait]
/// This represents the interface between a control and an action in SimplyDMX
pub trait ControllerServiceLink {
	/// Serializes information about the link so it can be saved to a file
	async fn save(&self) -> SerializedData;
	/// Clones the link so it can be applied to another control (namely for LiveAssign)
	async fn clone_link(&self) -> Box<dyn ControllerServiceLink + Send + Sync + 'static>;
	/// Handles an event emitted from the control
	async fn emit(&self, event: ControllerEvent);
	/// Gets what should be the current state of the control. This is used to initialize the
	/// control after creating a link.
	async fn get_current_value(&self) -> Arc<dyn ControlUpdate + Send + Sync + 'static>;
	/// Sets the channel that should be used for communicating updates to the control
	async fn set_update_channel(&self, update_channel: Sender<Arc<dyn ControlUpdate + Send + Sync + 'static>>);
	/// Unlinks the service from the control, tearing down any related interfaces
	async fn unlink(&self) {}
}

/// Describes what a controller's capabilities are. This could affect the heuristics
/// of the action it controls
pub enum ControlCapabilities {
	FaderColumn(FaderColumnCapabilities),
	Fader(FaderCapabilities),
	Knob(KnobCapabilities),
	Button(ButtonCapabilities),
}

/// Describes the capabilities of a fader column
pub struct FaderColumnCapabilities {
	pub fader: FaderCapabilities,
	pub button: ButtonCapabilities,
}

/// Describes the optional capabilities of a fader
pub struct FaderCapabilities {
	pub touch: bool,
}

/// Describes the optional capabilities of a knob
pub struct KnobCapabilities {
	pub push: bool,
}

/// Describes the optional capabilities of a button
pub struct ButtonCapabilities {
	pub velocity: bool,
}

/// An event emitted from a control. This top layer of the enum
/// tree indicates what kind of control emitted the event.
pub enum ControllerEvent {
	FaderColumn(ControllerFaderColumnEvent),
	Fader(ControllerFaderEvent),
	Knob(ControllerKnobEvent),
	Button(ControllerButtonEvent),
}

/// An event emitted by a fader column. This layer indicates which
/// item within the column emitted the event.
pub enum ControllerFaderColumnEvent {
	Fader(ControllerFaderEvent),
	Button(ControllerButtonEvent)
}

/// An event emitted by a fader.
pub enum ControllerFaderEvent {
	Pos(ScalableValue),
	Touch(bool),
}

/// An event emitted by a knob
pub enum ControllerKnobEvent {
	Pos(ScalableValue),
	Push(bool),
}

/// An event emitted by a button
pub enum ControllerButtonEvent {
	Push {
		state: bool,
		velocity: Option<ScalableValue>,
	}
}

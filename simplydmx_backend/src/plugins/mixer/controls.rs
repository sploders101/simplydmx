use std::sync::Arc;

use serde::{Serialize, Deserialize};
use super::MixerInterface;
use crate::{mixer_utils::{state::SubmasterData, static_layer::StaticLayer}, plugins::live_controller::{controller_services::{ControllerService, ControllerServiceLink, ControllerLinkError, ControllerRestoreError}, types::Control, scalable_value::ScalableValue}, utilities::{serialized_data::SerializedData, forms::FormDescriptor}};
use async_trait::async_trait;
use simplydmx_plugin_framework::*;

use uuid::Uuid;

// ┌──────────────────────────┐
// │    Layer Bin Commands    │
// └──────────────────────────┘

// #[interpolate_service(
// 	"enter_blind_mode",
// 	"Enter Blind Mode",
// 	"Copies the default layer bin to a new one with 0 opacity, setting it as the new default."
// )]
// impl EnterBlindMode {
// 	#![inner_raw(MixerInterface)]
// 	pub fn new(mixer_interface: MixerInterface) -> Self {
// 		Self(mixer_interface)
// 	}

// 	#[service_main()]
// 	async fn main(self) -> () {
// 		return self.0.enter_blind_mode().await;
// 	}
// }

// #[interpolate_service(
// 	"set_blind_opacity",
// 	"Set Blind Opacity",
// 	"Sets the opacity of the blind layer"
// )]
// impl SetBlindOpacity {
// 	#![inner_raw(MixerInterface)]
// 	pub fn new(mixer_interface: MixerInterface) -> Self {
// 		Self(mixer_interface)
// 	}

// 	#[service_main(
// 		("The desired opacity of the layer bin"),
// 	)]
// 	async fn main(self, opacity: u16) {
// 		return self.0.set_blind_opacity(opacity).await;
// 	}
// }

// #[interpolate_service(
// 	"revert_blind",
// 	"Revert blind changes",
// 	"Reverts all changes made in blind mode. Changes are made instantly. Use `set_blind_opacity` to fade.",
// )]
// impl RevertBlind {
// 	#![inner_raw(MixerInterface)]
// 	pub fn new(mixer_interface: MixerInterface) -> Self {
// 		Self(mixer_interface)
// 	}

// 	#[service_main()]
// 	async fn main(self) {
// 		return self.0.revert_blind().await;
// 	}
// }

// #[interpolate_service(
// 	"commit_blind",
// 	"Commit blind changes",
// 	"Commits all changes made in blind mode, deleting the previous look. Changes are made instantly. Use `set_blind_opacity` to fade.",
// )]
// impl CommitBlind {
// 	#![inner_raw(MixerInterface)]
// 	pub fn new(mixer_interface: MixerInterface) -> Self {
// 		Self(mixer_interface)
// 	}

// 	#[service_main()]
// 	async fn main(self) {
// 		return self.0.commit_blind().await;
// 	}
// }

// ┌──────────────────────┐
// │    Layer Commands    │
// └──────────────────────┘

// #[interpolate_service(
// 	"set_layer_opacity",
// 	"Set Layer Opacity",
// 	"Sets the opacity of a layer (Optionally within a specific bin)"
// )]
// impl SetLayerOpacity {
// 	#![inner_raw(MixerInterface)]
// 	pub fn new(mixer_interface: MixerInterface) -> Self {
// 		Self(mixer_interface)
// 	}

// 	#[service_main(
// 		("The UUID that identifies the submaster to be changed", "mixer::layer_id"),
// 		("The desired opacity, from 0 to 65535"),
// 		("Automatically insert if necessary when opacity > 0, and remove when opacity == 0"),
// 		("A boolean indicating if the opacity setting was successfully applied."),
// 	)]
// 	async fn main(self, submaster_id: Uuid, opacity: u16, auto_insert: bool) -> bool {
// 		return self
// 			.0
// 			.set_layer_opacity(submaster_id, opacity, auto_insert)
// 			.await;
// 	}
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Serializable compomnent of layer opacity controller links
pub struct LayerOpacityLink {
	submaster: Uuid,
}

/// A controller service for layer opacity
pub struct LayerOpacityControl(MixerInterface);
#[async_trait]
impl ControllerService for LayerOpacityControl {
	async fn get_form(
		&self,
		prefilled: Option<SerializedData>,
		control_group: Arc<Control>,
	) -> FormDescriptor {
		return FormDescriptor::new();
	}

	async fn create_link(
		&self,
		form_data: SerializedData,
		control_group: Arc<Control>,
	) -> Result<Arc<dyn ControllerServiceLink + Send + Sync + 'static>, ControllerLinkError> {
		let link_data: LayerOpacityLink = form_data.deserialize().map_err(|err| ControllerLinkError::BadFormData)?;
		match *control_group.as_ref() {
			Control::FaderColumn(ref fader_column) => {
				let mixer = self.0.clone();
				let link_data = link_data.clone();
				fader_column.capabilities.fader.position.set_analog_action(Some(Box::new(move |value: ScalableValue| {
					let mixer = mixer.clone();
					tokio::spawn(async move {
						mixer.set_layer_opacity(&link_data.submaster, value.into(), true).await;
					});
				})));
				if let Some(flash_btn) = fader_column.capabilities.flash_btn {
					flash_btn.push.set_bool_action(Some(Box::new(move |value: bool| {
						let mixer = mixer.clone();
						tokio::spawn(async move {
							mixer.flash_layer(&link_data.submaster, if value { Some(u16::MAX) } else { None }, true);
						});
					})));
				}
			}
			Control::Fader(ref fader) => {
				let mixer = self.0.clone();
				let link_data = link_data.clone();
				fader.capabilities.position.set_analog_action()
			}
		}
		return Ok(Arc::new((link_data, control_group)));
	}

	async fn load_from_save(
		&self,
		save_data: SerializedData,
		control_group: Arc<Control>,
	) -> Result<Arc<dyn ControllerServiceLink + Send + Sync + 'static>, ControllerRestoreError> {
		return self.create_link(save_data, control_group).await.map_err(|err| ControllerRestoreError::BadSaveData);
	}
}
#[async_trait]
impl ControllerServiceLink for (LayerOpacityLink, Arc<Control>) {
	async fn save(&self) -> SerializedData {
		return SerializedData::JSON(serde_json::to_value(&self.0).expect("Could not serialize LayerOpacityLink"));
	}
	async fn unlink(&self) {
		match *self.1.as_ref() {
			Control::FaderColumn(ref fader_column) => {
				fader_column.capabilities.fader.position.set_analog_action(None);
				if let Some(touch) = fader_column.capabilities.fader.touch {
					touch.set_bool_action(None);
				}
				if let Some(button) = fader_column.capabilities.flash_btn {
					button.push.set_bool_action(None);
				}
			}
			Control::Fader(ref fader) => {
				fader.capabilities.position.set_analog_action(None);
				if let Some(touch) = fader.capabilities.touch {
					touch.set_bool_action(None);
				}
			}
			Control::Knob(ref knob) => {
				knob.capabilities.position.set_analog_action(None);
				if let Some(push) = knob.capabilities.push {
					push.set_bool_action(None);
				}
			}
			Control::Button(ref button) => {
				button.capabilities.push.set_bool_action(None);
			}
		}
	}
}

// #[interpolate_service(
// 	"get_layer_opacity",
// 	"Get Layer Opacity",
// 	"Gets the opacity of a layer (Optionally within a specific bin)"
// )]
// impl GetLayerOpacity {
// 	#![inner_raw(MixerInterface)]
// 	pub fn new(mixer_interface: MixerInterface) -> Self {
// 		Self(mixer_interface)
// 	}

// 	#[service_main(
// 		("The UUID that identifies the submaster to be changed", "mixer::layer_id"),
// 		("The opacity if the layer from 0 to 65535. None or null if the submaster is not currently in the stack. (effective 0)"),
// 	)]
// 	async fn main(self, submaster_id: Uuid) -> Option<u16> {
// 		return self.0.get_layer_opacity(submaster_id).await;
// 	}
// }

// #[interpolate_service("delete_layer", "Delete Layer", "Deletes a layer from the registry")]
// impl DeleteLayer {
// 	#![inner_raw(MixerInterface)]
// 	pub fn new(mixer_interface: MixerInterface) -> Self {
// 		Self(mixer_interface)
// 	}

// 	#[service_main(
// 		("The ID of the submaster you would like to delete", "mixer::layer_id"),
// 		("Whether or not the layer existed"),
// 	)]
// 	async fn main(self, submaster_id: Uuid) -> bool {
// 		return self.0.delete_layer(submaster_id).await;
// 	}
// }

// #[interpolate_service(
// 	"request_blend",
// 	"Request Reblend",
// 	"Manually requests the mixer to blend layers and emit new output"
// )]
// impl RequestBlend {
// 	#![inner_raw(MixerInterface)]
// 	pub fn new(mixer_interface: MixerInterface) -> Self {
// 		Self(mixer_interface)
// 	}

// 	#[service_main()]
// 	fn main(self) {
// 		return self.0.request_blend();
// 	}
// }

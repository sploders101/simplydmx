use super::MixerInterface;
use crate::mixer_utils::{state::SubmasterData, static_layer::StaticLayer};
use simplydmx_plugin_framework::*;

use uuid::Uuid;

// ┌──────────────────────────┐
// │    Layer Bin Commands    │
// └──────────────────────────┘

#[interpolate_service(
	"enter_blind_mode",
	"Enter Blind Mode",
	"Copies the default layer bin to a new one with 0 opacity, setting it as the new default."
)]
impl EnterBlindMode {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main()]
	async fn main(self) -> () {
		return self.0.enter_blind_mode().await;
	}
}

#[interpolate_service(
	"set_blind_opacity",
	"Set Blind Opacity",
	"Sets the opacity of the blind layer"
)]
impl SetBlindOpacity {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main(
		("The desired opacity of the layer bin"),
	)]
	async fn main(self, opacity: u16) {
		return self.0.set_blind_opacity(opacity).await;
	}
}

#[interpolate_service(
	"get_blind_opacity",
	"Get Layer Bin Opacity",
	"Gets the opacity of the blind layer"
)]
impl GetBlindOpacity {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main(
		("The opacity of the blind layer bin. This will be None or null if blind mode is inactive"),
	)]
	async fn main(self) -> Option<u16> {
		return self.0.get_blind_opacity().await;
	}
}

#[interpolate_service(
	"revert_blind",
	"Revert blind changes",
	"Reverts all changes made in blind mode. Changes are made instantly. Use `set_blind_opacity` to fade.",
)]
impl RevertBlind {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main()]
	async fn main(self) {
		return self.0.revert_blind().await;
	}
}

#[interpolate_service(
	"commit_blind",
	"Commit blind changes",
	"Commits all changes made in blind mode, deleting the previous look. Changes are made instantly. Use `set_blind_opacity` to fade.",
)]
impl CommitBlind {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main()]
	async fn main(self) {
		return self.0.commit_blind().await;
	}
}

// ┌──────────────────────┐
// │    Layer Commands    │
// └──────────────────────┘

#[interpolate_service(
	"list_submasters",
	"List Submasters",
	"Lists all user-created layers (submasters)",
)]
impl ListSubmasters {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main(
		("An array of (id, name) tuples representing the submasters listed in the mixer"),
	)]
	async fn main(self) -> Vec<(Uuid, String)> {
		return self.0.list_submasters_with_names().await;
	}
}

#[interpolate_service(
	"create_layer",
	"Create Submaster",
	"Creates a new submaster that can be used for blending"
)]
impl CreateLayer {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main(
		("The name to assign to the submaster"),
		("UUID value that should be used from this point forward to identify the submaster", "mixer::layer_id"),
	)]
	async fn main(self, name: String) -> Uuid {
		return self.0.create_layer(name).await;
	}
}

#[interpolate_service(
	"rename_layer",
	"Rename Layer",
	"Renames a submaster"
)]
impl RenameLayer {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main(
		("The UUID of the submaster to rename"),
		("The new name for the submaster"),
	)]
	async fn main(self, submaster_id: Uuid, new_name: String) -> () {
		self.0.rename_layer(submaster_id, new_name).await;
	}
}

#[interpolate_service(
	"set_layer_contents",
	"Set Layer Contents",
	"Adds or removes content in a layer"
)]
impl SetLayerContents {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main(
		("UUID value to identify the submaster", "mixer::layer_id"),
		("Collection of values to merge with the existing submaster data"),
		("Boolean indicating whether or not the set was successful"),
	)]
	async fn main(self, submaster_id: Uuid, submaster_delta: SubmasterData) -> bool {
		return self
			.0
			.set_layer_contents(submaster_id, submaster_delta)
			.await;
	}
}

#[interpolate_service(
	"get_layer_contents",
	"Get Layer Contents",
	"Retrieves the contents of a layer"
)]
impl GetLayerContents {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main(
		("The UUID that identifies the submaster in question", "mixer::layer_id"),
		("The submaster's visible contents"),
	)]
	async fn main(self, submaster_id: Uuid) -> Option<StaticLayer> {
		return self.0.get_layer_contents(submaster_id).await;
	}
}

#[interpolate_service(
	"set_layer_opacity",
	"Set Layer Opacity",
	"Sets the opacity of a layer (Optionally within a specific bin)"
)]
impl SetLayerOpacity {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main(
		("The UUID that identifies the submaster to be changed", "mixer::layer_id"),
		("The desired opacity, from 0 to 65535"),
		("Automatically insert if necessary when opacity > 0, and remove when opacity == 0"),
		("A boolean indicating if the opacity setting was successfully applied."),
	)]
	async fn main(self, submaster_id: Uuid, opacity: u16, auto_insert: bool) -> bool {
		return self
			.0
			.set_layer_opacity(submaster_id, opacity, auto_insert)
			.await;
	}
}

#[interpolate_service(
	"get_layer_opacity",
	"Get Layer Opacity",
	"Gets the opacity of a layer (Optionally within a specific bin)"
)]
impl GetLayerOpacity {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main(
		("The UUID that identifies the submaster to be changed", "mixer::layer_id"),
		("The opacity if the layer from 0 to 65535. None or null if the submaster is not currently in the stack. (effective 0)"),
	)]
	async fn main(self, submaster_id: Uuid) -> Option<u16> {
		return self.0.get_layer_opacity(submaster_id).await;
	}
}

#[interpolate_service("delete_layer", "Delete Layer", "Deletes a layer from the registry")]
impl DeleteLayer {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main(
		("The ID of the submaster you would like to delete", "mixer::layer_id"),
		("Whether or not the layer existed"),
	)]
	async fn main(self, submaster_id: Uuid) -> bool {
		return self.0.delete_layer(submaster_id).await;
	}
}

#[interpolate_service(
	"request_blend",
	"Request Reblend",
	"Manually requests the mixer to blend layers and emit new output"
)]
impl RequestBlend {
	#![inner_raw(MixerInterface)]
	pub fn new(mixer_interface: MixerInterface) -> Self {
		Self(mixer_interface)
	}

	#[service_main()]
	fn main(self) {
		return self.0.request_blend();
	}
}

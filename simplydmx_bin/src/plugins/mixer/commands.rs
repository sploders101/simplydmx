use std::{
	sync::Arc,
	collections::HashMap,
};
use async_std::{
	channel::Sender,
	sync::Mutex,
};

use simplydmx_plugin_framework::*;
use super::state::{
	MixerContext,
	Submaster,
	LayerBin,
};
use super::blender::UpdateList;

use crate::type_extensions::{
	uuid::Uuid,
};


// ┌──────────────────────────┐
// │    Layer Bin Commands    │
// └──────────────────────────┘

#[interpolate_service(
	"enter_blind_mode",
	"Enter Blind Mode",
	"Copies the default layer bin to a new one with 0 opacity, setting it as the new default.",
)]
impl EnterBlindMode {
	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>) -> Self {
		return Self(plugin_context, mixer_context);
	}

	#[service_main(
		("UUID", "The UUID for the old layer bin. Returns None/null if blind mode is already active."),
	)]
	async fn main(self) -> Option<Uuid> {

		// Get resources
		let mut ctx = self.1.lock().await;

		if let Some(_) = ctx.transitional_layer_bin {
			return None;
		}

		let new_uuid = Uuid::new();

		// Clone default bin and insert
		let cloned_bin = LayerBin::clone(ctx.layer_bins.get(&ctx.default_layer_bin).unwrap());
		ctx.layer_bins.insert(new_uuid, cloned_bin);
		ctx.layer_bin_opacities.insert(new_uuid, 0);
		ctx.layer_bin_order.push(new_uuid);

		// Set new bin as default
		let old_layer_bin = ctx.default_layer_bin;
		ctx.default_layer_bin = new_uuid;

		// No blending needed as the opacity of all new data is zero.

		return Some(old_layer_bin);
	}
}

#[interpolate_service(
	"exit_blind_mode",
	"Exit blind mode",
	"Deletes the current layer bin setting it as the new default with full opacity`"
)]
impl ExitBlindMode {
	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc<Mutex<MixerContext>>) -> Self {
		return Self(plugin_context, mixer_context);
	}

	#[service_main()]
	async fn main(self) -> () {

		// Get resources
		let mut ctx = self.1.lock().await;

		// Delete layer bin and set transitional as default
		if let Some(blind_bin) = ctx.transitional_layer_bin {
			let old_layer_bin = ctx.default_layer_bin;
			ctx.layer_bins.remove(&old_layer_bin);
			ctx.layer_bin_order.retain(|&x| x != old_layer_bin);
			ctx.default_layer_bin = blind_bin.clone();
			ctx.transitional_layer_bin = None;
			ctx.layer_bin_opacities.insert(blind_bin, u16::MAX);
		}

	}
}

// **** Incomplete **** //
#[interpolate_service(
	"set_blind_opacity",
	"Set Blind Opacity",
	"Sets the opacity of the blind layer",
)]
impl SetBlindOpacity {
	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>, Sender::<UpdateList>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>, update_sender: Sender<UpdateList>) -> Self {
		return Self(plugin_context, mixer_context, update_sender);
	}

	#[service_main(
		("UUID", "The UUID of the layer bin you would like to change", "mixer::layer_bin"),
		("Opacity", "The desired opacity of the layer bin"),
	)]
	async fn main(self, uuid: Uuid, opacity: u16) {
		let mut ctx = self.1.lock().await;
		ctx.layer_bin_opacities.insert(uuid, opacity);
	}
}

// **** Incomplete **** //
#[interpolate_service(
	"get_blind_opacity",
	"Get Layer Bin Opacity",
	"Gets the opacity of the blind layer",
)]
impl GetBlindOpacity {
	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>) -> Self {
		return Self(plugin_context, mixer_context);
	}

	#[service_main()]
	async fn main(self) {
	}
}

// **** Incomplete **** //
#[interpolate_service(
	"revert_blind",
	"Revert blind changes",
	"Reverts all changes made in blind mode. Changes are made instantly. Use `set_blind_opacity` to fade.",
)]
impl RevertBlind {
	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>, Sender::<UpdateList>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>, update_sender: Sender<UpdateList>) -> Self {
		return Self(plugin_context, mixer_context, update_sender);
	}

	#[service_main()]
	async fn main(self) {
	}
}


// **** Incomplete **** //
#[interpolate_service(
	"commit_blind",
	"Commit blind changes",
	"Commits all changes made in blind mode, deleting the previous look. Changes are made instantly. Use `set_blind_opacity` to fade.",
)]
impl CommitBlind {
	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>, Sender::<UpdateList>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>, update_sender: Sender<UpdateList>) -> Self {
		return Self(plugin_context, mixer_context, update_sender);
	}

	#[service_main()]
	async fn main(self) {
	}
}


// ┌──────────────────────┐
// │    Layer Commands    │
// └──────────────────────┘

#[interpolate_service(
	"create_layer",
	"Create Submaster",
	"Creates a new submaster that can be used for blending",
)]
impl CreateLayer {

	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>, Sender::<UpdateList>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>, update_sender: Sender<UpdateList>) -> Self {
		return Self(plugin_context, mixer_context, update_sender);
	}

	#[service_main(
		("Submaster Identifier", "UUID value that should be used from this point forward to identify the submaster"),
	)]
	async fn main(self) -> Uuid {
		let uuid = Uuid::new();
		let mut context = self.1.lock().await;

		context.submasters.insert(uuid, Submaster {
			data: HashMap::new(),
		});

		return uuid;
	}
}

// **** Incomplete **** //
#[interpolate_service(
	"set_layer_contents",
	"Set Layer Contents",
	"Adds or removes content in a layer",
)]
impl SetLayerContents {
	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>, Sender::<UpdateList>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>, update_sender: Sender<UpdateList>) -> Self {
		return Self(plugin_context, mixer_context, update_sender);
	}

	#[service_main()]
	async fn main(self) {
	}
}

// **** Incomplete **** //
#[interpolate_service(
	"get_layer_contents",
	"Get Layer Contents",
	"Retrieves the contents of a layer",
)]
impl GetLayerContents {
	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>) -> Self {
		return Self(plugin_context, mixer_context);
	}

	#[service_main()]
	async fn main(self) {
	}
}

// **** Incomplete **** //
#[interpolate_service(
	"set_layer_opacity",
	"Set Layer Opacity",
	"Sets the opacity of a layer (Optionally within a specific bin)",
)]
impl SetLayerOpacity {
	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>, Sender::<UpdateList>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>, update_sender: Sender<UpdateList>) -> Self {
		return Self(plugin_context, mixer_context, update_sender);
	}

	#[service_main()]
	async fn main(self) {
	}
}

// **** Incomplete **** //
#[interpolate_service(
	"get_layer_opacity",
	"Get Layer Opacity",
	"Gets the opacity of a layer (Optionally within a specific bin)",
)]
impl GetLayerOpacity {
	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>) -> Self {
		return Self(plugin_context, mixer_context);
	}

	#[service_main()]
	async fn main(self) {
	}
}

// **** Incomplete **** //
#[interpolate_service(
	"delete_layer",
	"Delete Layer",
	"Deletes a layer from the registry",
)]
impl DeleteLayer {
	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>, Sender::<UpdateList>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>, update_sender: Sender<UpdateList>) -> Self {
		return Self(plugin_context, mixer_context, update_sender);
	}

	#[service_main()]
	async fn main(self) {
	}
}

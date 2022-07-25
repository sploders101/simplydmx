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
	SubmasterDelta,
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
		("Layer Bin ID", "The UUID for the old layer bin. Returns None/null if blind mode is already active.", "mixer::layer_bin_id"),
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
		("Opacity", "The desired opacity of the layer bin"),
	)]
	async fn main(self, opacity: u16) {
		let mut ctx = self.1.lock().await;
		if ctx.transitional_layer_bin.is_some() {
			let uuid = ctx.default_layer_bin;
			ctx.layer_bin_opacities.insert(uuid, opacity);
			// Sender can only fail if we are shutting down, in which case we don't care if this fails
			self.2.send(UpdateList::LayerBin).await.ok();
		}
	}
}

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

	#[service_main(
		("Blind Opacity", "The opacity of the blind layer bin. This will be None or null if blind mode is inactive"),
	)]
	async fn main(self) -> Option<u16> {
		let ctx = self.1.lock().await;
		if ctx.transitional_layer_bin.is_some() {
			if let Some(opacity) = ctx.layer_bin_opacities.get(&ctx.default_layer_bin) {
				return Some(u16::clone(opacity));
			} else {
				return None;
			}
		} else {
			return None;
		}
	}
}

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
		// Get resources
		let mut ctx = self.1.lock().await;

		// Delete layer bin and set transitional as default
		if let Some(old_bin) = ctx.transitional_layer_bin {
			let blind_bin = ctx.default_layer_bin;
			ctx.layer_bins.remove(&blind_bin);
			ctx.layer_bin_order.retain(|&x| x != blind_bin);
			ctx.default_layer_bin = old_bin;
			ctx.transitional_layer_bin = None;
			ctx.layer_bin_opacities.insert(old_bin, u16::MAX);
			self.2.send(UpdateList::LayerBin).await.ok();
		}
	}
}


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
		// Get resources
		let mut ctx = self.1.lock().await;

		// Delete transitional layer bin
		if let Some(old_bin) = ctx.transitional_layer_bin {
			let blind_bin = ctx.default_layer_bin;
			ctx.layer_bins.remove(&old_bin);
			ctx.layer_bin_order.retain(|&x| x != old_bin);
			ctx.transitional_layer_bin = None;
			ctx.layer_bin_opacities.insert(blind_bin, u16::MAX);
			self.2.send(UpdateList::LayerBin).await.ok();
		}
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

	#![inner_raw(PluginContext, Arc::<Mutex::<MixerContext>>)]

	pub fn new(plugin_context: PluginContext, mixer_context: Arc::<Mutex::<MixerContext>>) -> Self {
		return Self(plugin_context, mixer_context);
	}

	#[service_main(
		("Submaster ID", "UUID value that should be used from this point forward to identify the submaster", "mixer::layer_id"),
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

	#[service_main(
		("Submaster ID", "UUID value to identify the submaster", "mixer::layer_id"),
		("Submaster Delta", "Collection of values to merge with the existing submaster data"),
		("Success", "Boolean indicating whether or not the set was successful"),
	)]
	async fn main(self, uuid: Uuid, submaster_delta: SubmasterDelta) -> bool {
		let mut ctx = self.1.lock().await;

		// Check if the specified submaster exists
		if let Some(submaster) = ctx.submasters.get_mut(&uuid) {
			// Loop through fixtures in the delta
			for (fixture_id, fixture_data) in submaster_delta.iter() {

				// If fixture doesn't exist in the submaster, create it, then get the mutable data
				if !submaster.data.contains_key(fixture_id) {
					submaster.data.insert(fixture_id.clone(), HashMap::new());
				}
				let current_fixture_data = submaster.data.get_mut(fixture_id).unwrap();

				// Loop through attributes in the delta
				for (attribute_id, attribute_value) in fixture_data.iter() {

					if let Some(attribute_value) = attribute_value {
						current_fixture_data.insert(attribute_id.clone(), attribute_value.clone());
					} else {
						current_fixture_data.remove(attribute_id);
					}

				}

				// TODO: ST: Emit `mixer.submaster_content`
				// TODO: LT: Remove unused fixture maps so we don't loop through them unnecessarily

			}
			self.2.send(UpdateList::Submaster(uuid)).await.ok();
			return true;
		} else {
			return false;
		}
	}
}

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

	#[service_main(
		("Submaster ID", "The UUID that identifies the submaster in question", "mixer::layer_id"),
		("Submaster Data", "The submaster's visible contents"),
	)]
	async fn main(self, uuid: Uuid) -> Option::<Submaster> {
		let ctx = self.1.lock().await;
		// TODO: ST: Maybe use Arc instead of cloning?
		return match ctx.submasters.get(&uuid) {
			Some(submaster) => Some(submaster.clone()),
			None => None,
		}
	}
}

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

	#[service_main(
		("Submaster ID", "The UUID that identifies the submaster to be changed", "mixer::layer_id"),
		("Opacity", "The desired opacity, from 0 to 65535"),
		("Automatic insertion", "Automatically insert if necessary when opacity > 0, and remove when opacity == 0"),
		("Layer Bin", "ID of the desired layer bin to make the change on. Default is used if not provided."),
		("Success", "A boolean indicating if the opacity setting was successfully applied."),
	)]
	async fn main(self, uuid: Uuid, opacity: u16, auto_insert: bool, layer_bin_id: Option::<Uuid>) -> bool {
		let mut ctx = self.1.lock().await;
		let default_layer_bin = ctx.default_layer_bin;
		if let Some(layer_bin) = ctx.layer_bins.get_mut(&layer_bin_id.unwrap_or(default_layer_bin)) {
			if auto_insert {
				if opacity > 0 && !layer_bin.layer_order.contains(&uuid) {
					layer_bin.layer_order.push(uuid.clone());
				} else if opacity == 0 && layer_bin.layer_order.contains(&uuid) {
					layer_bin.layer_order.retain(|x| x != &uuid);
				}
			}
			layer_bin.layer_opacities.insert(uuid, opacity);
			self.2.send(UpdateList::Submaster(uuid)).await.ok();
			return true;
		} else {
			return false;
		}
	}
}

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

	#[service_main(
		("Submaster ID", "The UUID that identifies the submaster to be changed", "mixer::layer_id"),
		("Layer Bin ID", "The UUID of the layer bin you would like to query. If None/null, the default bin will be used."),
		("Layer Opacity", "The opacity if the layer from 0 to 65535. None or null if the submaster is not currently in the stack. (effective 0)"),
	)]
	async fn main(self, uuid: Uuid, layer_bin_id: Option::<Uuid>) -> Option::<u16> {
		let ctx = self.1.lock().await;
		if let Some(layer_bin) = ctx.layer_bins.get(&layer_bin_id.unwrap_or(ctx.default_layer_bin)) {
			if layer_bin.layer_order.contains(&uuid) {
				if let Some(opacity) = layer_bin.layer_opacities.get(&uuid) {
					return Some(opacity.clone());
				} else {
					return None;
				}
			} else {
				return None;
			}
		} else {
			return None;
		}
	}
}

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

	#[service_main(
		("Submaster ID", "The ID of the submaster you would like to delete", "mixer::layer_id"),
		("Existed", "Whether or not the layer existed"),
	)]
	async fn main(self, uuid: Uuid) -> bool {
		let mut ctx = self.1.lock().await;

		// Remove submaster
		let was_removed = ctx.submasters.remove(&uuid).is_some();

		// Remove references
		for layer_bin in ctx.layer_bins.values_mut() {
			layer_bin.layer_order.retain(|x| x != &uuid);
			layer_bin.layer_opacities.remove(&uuid);
		}

		// Signal to update everything and return
		self.2.send(UpdateList::All).await.ok();
		return was_removed;
	}
}

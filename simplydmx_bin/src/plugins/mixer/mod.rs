mod commands;
mod state;
mod blender;

/// Exported types may be used by other plugins when communicating with the mixer
pub mod exported_types {
	pub use super::state::{
		Submaster,
		AbstractLayerLight,
		BlenderValue,
		FullMixerOutput,
		FixtureMixerOutput,
		BlendingScheme,
		FullMixerBlendingData,
		SnapData,
		BlendingData,
	};
}

use std::{
	sync::Arc,
	collections::HashMap,
};
use async_std::{
	sync::Mutex,
	channel::Sender,
};
use async_trait::async_trait;
use simplydmx_plugin_framework::*;

use state::{
	MixerContext,
	FullMixerOutput,
	Submaster,
};
use uuid::Uuid;

use self::{
	blender::UpdateList,
	state::{
		LayerBin,
		SubmasterDelta,
	},
};

use super::{
	patcher::PatcherInterface,
	saver::{
		SaverInterface,
		Savable,
	},
};

pub async fn initialize_mixer(plugin_context: PluginContext, saver: SaverInterface, patcher: PatcherInterface) -> Result<MixerInterface, MixerInitializationError> {

	// Create mixer context
	let mixer_context = Arc::new(Mutex::new(if let Ok(data) = saver.load_data(&"mixer".into()).await {
		if let Some(data) = data {
			MixerContext::from_file(data)
		} else {
			MixerContext::new()
		}
	} else {
		return Err(MixerInitializationError::UnrecognizedData);
	}));


	// Declare events

	plugin_context.declare_event::<FullMixerOutput>(
		"mixer.layer_bin_output".to_owned(),
		Some("Emitted when a layer bin has been updated in the mixer".to_owned()),
	).await.unwrap();

	plugin_context.declare_event::<FullMixerOutput>(
		"mixer.final_output".to_owned(),
		Some("Emitted when the mixer's final output has been updated".to_owned()),
	).await.unwrap();


	// Start blender task
	let update_sender = blender::start_blender(plugin_context.clone(), Arc::clone(&mixer_context), patcher).await;

	// Send kickstart to blender task to recover any data that was saved
	update_sender.send(UpdateList::All).await.unwrap();

	// Create mixer interface
	let interface = MixerInterface::new(plugin_context.clone(), mixer_context, update_sender);

	// Register services
	plugin_context.register_service(true, commands::EnterBlindMode::new(interface.clone())).await.unwrap();
	plugin_context.register_service(true, commands::SetBlindOpacity::new(interface.clone())).await.unwrap();
	plugin_context.register_service(true, commands::GetBlindOpacity::new(interface.clone())).await.unwrap();
	plugin_context.register_service(true, commands::RevertBlind::new(interface.clone())).await.unwrap();
	plugin_context.register_service(true, commands::CommitBlind::new(interface.clone())).await.unwrap();
	plugin_context.register_service(true, commands::CreateLayer::new(interface.clone())).await.unwrap();
	plugin_context.register_service(true, commands::SetLayerContents::new(interface.clone())).await.unwrap();
	plugin_context.register_service(true, commands::GetLayerContents::new(interface.clone())).await.unwrap();
	plugin_context.register_service(true, commands::SetLayerOpacity::new(interface.clone())).await.unwrap();
	plugin_context.register_service(true, commands::GetLayerOpacity::new(interface.clone())).await.unwrap();
	plugin_context.register_service(true, commands::DeleteLayer::new(interface.clone())).await.unwrap();

	// Register saving mechanism
	saver.register_savable("mixer", interface.clone()).await.unwrap();

	return Ok(interface);
}

#[derive(Clone)]
pub struct MixerInterface(PluginContext, Arc<Mutex<MixerContext>>, Sender::<UpdateList>);
impl MixerInterface {
	pub fn new(plugin_context: PluginContext, mixer_context: Arc<Mutex<MixerContext>>, update_sender: Sender::<UpdateList>) -> Self {
		return Self(plugin_context, mixer_context, update_sender);
	}

	/// Copies the default layer bin to a new one with 0 opacity, setting it as the new default.
	///
	/// Returns The UUID for the old layer bin. Returns None/null if blind mode is already active.
	async fn enter_blind_mode(&self) -> Option<Uuid> {

		// Get resources
		let mut ctx = self.1.lock().await;

		if let Some(_) = ctx.transitional_layer_bin {
			return None;
		}

		let new_uuid = Uuid::new_v4();

		// Clone default bin and insert
		let cloned_bin = LayerBin::clone(ctx.layer_bins.get(&ctx.default_layer_bin).unwrap());
		ctx.layer_bins.insert(new_uuid, cloned_bin);
		ctx.layer_bin_opacities.insert(new_uuid, 0);
		ctx.layer_bin_order.push(new_uuid);

		// Set new bin as default
		let old_layer_bin = ctx.default_layer_bin;
		ctx.default_layer_bin = new_uuid;
		ctx.transitional_layer_bin = Some(old_layer_bin.clone());

		// No blending needed as the opacity of all new data is zero.

		return Some(old_layer_bin);
	}

	/// Sets the opacity of the blind layer
	async fn set_blind_opacity(&self, opacity: u16) {
		let mut ctx = self.1.lock().await;
		if ctx.transitional_layer_bin.is_some() {
			let blind_bin_id = ctx.default_layer_bin;
			ctx.layer_bin_opacities.insert(blind_bin_id, opacity);
			// Sender can only fail if we are shutting down, in which case we don't care if this fails
			self.2.send(UpdateList::LayerBin).await.ok();
		}
	}

	/// Gets the opacity of the blind layer
	/// Returns the opacity of the blind layer bin. This will be None or null if blind mode is inactive
	async fn get_blind_opacity(&self) -> Option<u16> {
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

	/// Reverts all changes made in blind mode. Changes are made instantly. Use `set_blind_opacity` to fade.
	async fn revert_blind(&self) {
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

	/// Commits all changes made in blind mode, deleting the previous look. Changes are made instantly. Use `set_blind_opacity` to fade.
	async fn commit_blind(&self) {
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

	/// Creates a new submaster that can be used for blending
	///
	/// Returns the ID of the new submaster
	async fn create_layer(&self) -> Uuid {
		let submaster_id = Uuid::new_v4();
		#[cfg(feature = "verbose-debugging")]
		println!("Aquiring lock in create_layer");
		let mut context = self.1.lock().await;

		context.submasters.insert(submaster_id, Submaster {
			data: HashMap::new(),
		});

		#[cfg(feature = "verbose-debugging")]
		println!("Dropping lock in create_layer");
		return submaster_id;
	}

	/// Adds or removes content in a layer
	async fn set_layer_contents(&self, submaster_id: Uuid, submaster_delta: SubmasterDelta) -> bool {
		#[cfg(feature = "verbose-debugging")]
		println!("Aquiring lock in set_layer_contents");
		let mut ctx = self.1.lock().await;

		// Check if the specified submaster exists
		if let Some(submaster) = ctx.submasters.get_mut(&submaster_id) {
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
			self.2.send(UpdateList::Submaster(submaster_id)).await.ok();
			#[cfg(feature = "verbose-debugging")]
			println!("Dropping lock in set_layer_contents");
			return true;
		} else {
			#[cfg(feature = "verbose-debugging")]
			println!("Dropping lock in set_layer_contents");
			return false;
		}
	}

	/// Retrieves the contents of a layer
	async fn get_layer_contents(&self, submaster_id: Uuid) -> Option::<Submaster> {
		let ctx = self.1.lock().await;
		// TODO: ST: Maybe use Arc instead of cloning?
		return match ctx.submasters.get(&submaster_id) {
			Some(submaster) => Some(submaster.clone()),
			None => None,
		}
	}

	/// Sets the opacity of a layer (Optionally within a specific bin)
	///
	/// If auto-insert is true, the layer will be automatically inserted if `opacity > 0` and it isn't in the stack.
	/// Likewise, it will be removed if `opacity == 0` and it *is* in the stack
	///
	/// Returns a boolean indicating if the operation was successful (this can be safely ignored)
	async fn set_layer_opacity(&self, submaster_id: Uuid, opacity: u16, auto_insert: bool, layer_bin_id: Option::<Uuid>) -> bool {
		#[cfg(feature = "verbose-debugging")]
		println!("Aquiring lock in set_layer_opacity");
		let mut ctx = self.1.lock().await;
		let default_layer_bin = ctx.default_layer_bin;
		if let Some(layer_bin) = ctx.layer_bins.get_mut(&layer_bin_id.unwrap_or(default_layer_bin)) {
			if auto_insert {
				if opacity > 0 && !layer_bin.layer_order.contains(&submaster_id) {
					layer_bin.layer_order.push(submaster_id.clone());
				} else if opacity == 0 && layer_bin.layer_order.contains(&submaster_id) {
					layer_bin.layer_order.retain(|x| *x != submaster_id);
				}
			}
			layer_bin.layer_opacities.insert(submaster_id, opacity);
			if opacity == 0 && auto_insert {
				self.2.send(UpdateList::All).await.ok();
			} else {
				self.2.send(UpdateList::Submaster(submaster_id)).await.ok();
			}
			#[cfg(feature = "verbose-debugging")]
			println!("Dopping lock in set_layer_opacity");
			return true;
		} else {
			#[cfg(feature = "verbose-debugging")]
			println!("Dopping lock in set_layer_opacity");
			return false;
		}
	}

	/// Gets the opacity of a layer (Optionally within a specific bin)
	///
	/// Returns the opacity of the layer, or `None` if it is not in the stack
	async fn get_layer_opacity(&self, submaster_id: Uuid, layer_bin_id: Option::<Uuid>) -> Option::<u16> {
		let ctx = self.1.lock().await;
		if let Some(layer_bin) = ctx.layer_bins.get(&layer_bin_id.unwrap_or(ctx.default_layer_bin)) {
			if layer_bin.layer_order.contains(&submaster_id) {
				if let Some(opacity) = layer_bin.layer_opacities.get(&submaster_id) {
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

	/// Deletes a layer from the registry
	///
	/// Returns a boolean indicating if the operation was successful (this can be safely ignored).
	async fn delete_layer(&self, submaster_id: Uuid) -> bool {
		let mut ctx = self.1.lock().await;

		// Remove submaster
		let was_removed = ctx.submasters.remove(&submaster_id).is_some();

		// Remove references
		for layer_bin in ctx.layer_bins.values_mut() {
			layer_bin.layer_order.retain(|x| x != &submaster_id);
			layer_bin.layer_opacities.remove(&submaster_id);
		}

		// Signal to update everything and return
		self.2.send(UpdateList::All).await.ok();
		return was_removed;
	}

}

#[async_trait]
impl Savable for MixerInterface {
	async fn save_data(&self) -> Result<Option<Vec<u8>>, String> {
		return Ok(Some(self.1.lock().await.serialize_cbor()?));
	}
}

#[portable]
#[derive(Debug)]
pub enum MixerInitializationError {
	UnrecognizedData,
}

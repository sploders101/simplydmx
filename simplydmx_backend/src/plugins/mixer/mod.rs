mod blender;
mod commands;
mod state;

use super::{
	patcher::PatcherInterface,
	saver::{Savable, SaverInterface},
};
use crate::mixer_utils::{
	state::{BlenderValue, FullMixerOutput, SubmasterData},
	static_layer::StaticLayer,
};
use async_trait::async_trait;
use rustc_hash::FxHashMap;
use simplydmx_plugin_framework::*;
use state::MixerContext;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{RwLock, Notify};
use uuid::Uuid;

pub async fn initialize_mixer(
	plugin_context: PluginContext,
	saver: SaverInterface,
	patcher: PatcherInterface,
) -> Result<MixerInterface, MixerInitializationError> {
	// Create mixer context
	let mixer_context = Arc::new(RwLock::new(
		if let Ok(data) = saver.load_data(&"mixer".into()).await {
			if let Some(data) = data {
				MixerContext::from_file(data)
			} else {
				MixerContext::new()
			}
		} else {
			return Err(MixerInitializationError::UnrecognizedData);
		},
	));

	// Declare events

	plugin_context
		.declare_event::<()>(
			"mixer.blind".into(),
			Some("Emitted when blind mode is enabled or disabled".into()),
		)
		.await
		.unwrap();

	plugin_context
		.declare_event::<()>(
			"mixer.new_submaster".into(),
			Some("Emitted when a new submaster is created".into()),
		)
		.await
		.unwrap();

	plugin_context
		.declare_event::<()>(
			"mixer.submaster_renamed".into(),
			Some("Emitted when a submaster is renamed".into()),
		)
		.await
		.unwrap();

	plugin_context
		.declare_event::<SubmasterData>(
			"mixer.submaster_updated".into(),
			Some("Emitted when a submaster is changed. Filter is a UUID of the submaster that was changed".into()),
		)
		.await
		.unwrap();

	plugin_context
		.declare_event::<FullMixerOutput>(
			"mixer.final_output".into(),
			Some("Emitted when the mixer's final output has been updated".into()),
		)
		.await
		.unwrap();

	// Start blender task
	let update_sender =
		blender::start_blender(plugin_context.clone(), Arc::clone(&mixer_context), patcher).await;

	// Send kickstart to blender task to recover any data that was saved
	// TODO: Verify this is no longer needed due to the switch from wait-then-run rate-limiting to run-then-wait
	// update_sender.send(UpdateList::All).await.unwrap();

	// Create mixer interface
	let interface = MixerInterface::new(plugin_context.clone(), mixer_context, update_sender);

	// Register services
	plugin_context
		.register_service(true, commands::EnterBlindMode::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::SetBlindOpacity::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::GetBlindOpacity::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::RevertBlind::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::CommitBlind::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::CreateLayer::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::RenameLayer::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::ListSubmasters::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::SetLayerContents::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::GetLayerContents::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::SetLayerOpacity::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::GetLayerOpacity::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::DeleteLayer::new(interface.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, commands::RequestBlend::new(interface.clone()))
		.await
		.unwrap();

	// Register type specifiers
	plugin_context
		.register_service_type_specifier(
			"submasters".into(),
			SubmasterTypeSpecifier(interface.clone()),
		)
		.await
		.unwrap();

	// Register saving mechanism
	saver
		.register_savable("mixer", interface.clone())
		.await
		.unwrap();

	return Ok(interface);
}

#[derive(Clone)]
pub struct MixerInterface(PluginContext, Arc<RwLock<MixerContext>>, Arc<Notify>);
impl MixerInterface {
	pub fn new(
		plugin_context: PluginContext,
		mixer_context: Arc<RwLock<MixerContext>>,
		update_sender: Arc<Notify>,
	) -> Self {
		return Self(plugin_context, mixer_context, update_sender);
	}

	/// Copies the default layer bin to a new one with 0 opacity, setting it as the new default.
	///
	/// Returns The UUID for the old layer bin. Returns None/null if blind mode is already active.
	pub async fn enter_blind_mode(&self) -> () {
		// Get resources
		let mut ctx = self.1.write().await;

		if ctx.frozen_context.is_none() {
			ctx.blind_opacity = 0;
			ctx.frozen_context = Some(ctx.default_context.clone());

			self.0
				.emit("mixer.blind".into(), FilterCriteria::None, true)
				.await;
			// self.2.send(()).await; // Not needed because changes would not be visible
		}
	}

	/// Sets the opacity of the blind layer
	pub async fn set_blind_opacity(&self, opacity: u16) {
		let mut ctx = self.1.write().await;

		if ctx.frozen_context.is_some() {
			ctx.blind_opacity = opacity;
			self.2.notify_one();
		}
	}

	/// Gets the opacity of the blind layer
	/// Returns the opacity of the blind layer bin. This will be None or null if blind mode is inactive
	pub async fn get_blind_opacity(&self) -> Option<u16> {
		let ctx = self.1.read().await;

		if ctx.frozen_context.is_some() {
			return Some(ctx.blind_opacity);
		} else {
			return None;
		}
	}

	/// Reverts all changes made in blind mode. Changes are made instantly.
	/// Use `set_blind_opacity` to fade.
	pub async fn revert_blind(&self) {
		// Get resources
		let mut ctx = self.1.write().await;

		if let Some(mixing_context) = ctx.frozen_context.take() {
			ctx.default_context = mixing_context;
			ctx.blind_opacity = 0;
			self.2.notify_one();
		}
	}

	/// Commits all changes made in blind mode, deleting the previous look.
	/// Changes are made instantly. Use `set_blind_opacity` to fade.
	pub async fn commit_blind(&self) {
		// Get resources
		let mut ctx = self.1.write().await;

		if ctx.frozen_context.is_some() {
			ctx.frozen_context = None;
			ctx.blind_opacity = 0;
			self.2.notify_one();
		}
	}

	/// Lists all static layers (submasters)
	pub async fn list_submasters(&self) -> Vec<Uuid> {
		let ctx = self.1.read().await;
		return ctx.default_context.user_submaster_order.clone();
	}

	/// Lists all static layers (submasters) with names
	pub async fn list_submasters_with_names(&self) -> Vec<(Uuid, String)> {
		let ctx = self.1.read().await;
		return ctx
			.default_context
			.user_submaster_order
			.iter()
			.map(|id| {
				(
					id.clone(),
					if let Some(submaster) = ctx.default_context.user_submasters.get(id) {
						submaster.name.clone()
					} else {
						"ERROR: Broken submaster reference".into()
					},
				)
			})
			.collect();
	}

	/// Creates a new submaster that can be used for blending
	///
	/// Returns the ID of the new submaster
	pub async fn create_layer(&self, name: String) -> Uuid {
		let mut ctx = self.1.write().await;
		let submaster_id = Uuid::new_v4();

		ctx.default_context
			.user_submasters
			.insert(submaster_id.clone(), StaticLayer::new(name));
		ctx.default_context.user_submaster_order.push(submaster_id.clone());

		self.0
			.emit(
				"mixer.new_submaster".into(),
				FilterCriteria::None,
				submaster_id.clone(),
			)
			.await;

		return submaster_id;
	}

	/// Renames a layer in the mixer
	pub async fn rename_layer(&self, submaster_id: Uuid, new_name: String) -> () {
		let mut ctx = self.1.write().await;

		if let Some(ref mut blind_context) = ctx.frozen_context {
			if let Some(submaster) = blind_context.user_submasters.get_mut(&submaster_id) {
				submaster.name = new_name.clone();
			}
		}
		if let Some(submaster) = ctx.default_context.user_submasters.get_mut(&submaster_id) {
			submaster.name = new_name;
		}

		self.0
			.emit(
				"mixer.submaster_renamed".into(),
				FilterCriteria::None,
				submaster_id.clone(),
			)
			.await;
	}

	/// Adds or removes content in a layer
	pub async fn set_layer_contents(&self, submaster_id: Uuid, submaster_delta: SubmasterData) -> bool {
		let mut ctx = self.1.write().await;

		// Check if the specified submaster exists
		if let Some(submaster) = ctx.default_context.user_submasters.get_mut(&submaster_id) {
			// Loop through fixtures in the delta
			for (fixture_id, fixture_data) in submaster_delta.iter() {
				// If fixture doesn't exist in the submaster, create it, then get the mutable data
				if !submaster.values.contains_key(fixture_id) {
					submaster.values.insert(fixture_id.clone(), FxHashMap::default());
				}
				let current_fixture_data = submaster.values.get_mut(fixture_id).unwrap();

				// Loop through attributes in the delta
				for (attribute_id, attribute_value) in fixture_data.iter() {
					match attribute_value {
						BlenderValue::None => {
							current_fixture_data.remove(attribute_id);
						}
						value => {
							current_fixture_data.insert(attribute_id.clone(), value.clone());
						}
					}
				}
			}

			// Emit the change for the frontend
			self.0.emit(
				"mixer.submaster_updated".into(),
				FilterCriteria::Uuid(submaster_id.clone()),
				submaster_delta,
			).await;

			if let Some(opacity) = ctx.default_context.layer_opacities.get(&submaster_id) {
				if *opacity > 0 {
					self.2.notify_one();
				}
			}
			return true;
		} else {
			return false;
		}
	}

	/// Retrieves the contents of a layer
	pub async fn get_layer_contents(&self, submaster_id: Uuid) -> Option<StaticLayer> {
		let ctx = self.1.read().await;
		return ctx
			.default_context
			.user_submasters
			.get(&submaster_id)
			.cloned();
	}

	/// Sets the opacity of a layer (Optionally within a specific bin)
	///
	/// If auto-insert is true, the layer will be automatically inserted if `opacity > 0` and it isn't in the stack.
	/// Likewise, it will be removed if `opacity == 0` and it *is* in the stack
	///
	/// Returns a boolean indicating if the operation was successful (this can be safely ignored)
	pub async fn set_layer_opacity(&self, submaster_id: Uuid, opacity: u16, auto_insert: bool) -> bool {
		let mut ctx = self.1.write().await;
		if ctx
			.default_context
			.user_submasters
			.contains_key(&submaster_id)
		{
			ctx.default_context
				.layer_opacities
				.insert(submaster_id, opacity);
			if auto_insert {
				if opacity > 0 && !ctx.default_context.layer_order.contains(&submaster_id) {
					ctx.default_context.layer_order.push(submaster_id.clone())
				} else if opacity == 0 && ctx.default_context.layer_order.contains(&submaster_id) {
					ctx.default_context
						.layer_order
						.retain(|x| *x != submaster_id);
				}
			}
			// TODO: Send this event only if the opacity *changes*
			self.2.notify_one();
			return true;
		} else {
			return false;
		}
	}

	/// Gets the opacity of a layer (Optionally within a specific bin)
	///
	/// Returns the opacity of the layer, or `None` if it is not in the stack
	pub async fn get_layer_opacity(&self, submaster_id: Uuid) -> Option<u16> {
		let ctx = self.1.read().await;
		return match ctx.default_context.layer_opacities.get(&submaster_id) {
			Some(opacity) => Some(*opacity),
			None => None,
		};
	}

	/// Deletes a layer from the registry
	///
	/// Returns a boolean indicating if the operation was successful (this can be safely ignored).
	pub async fn delete_layer(&self, submaster_id: Uuid) -> bool {
		let mut ctx = self.1.write().await;

		// Remove submaster
		let was_removed = ctx
			.default_context
			.user_submasters
			.remove(&submaster_id)
			.is_some();

		// Remove references
		ctx.default_context
			.layer_order
			.retain(|x| x != &submaster_id);
		ctx.default_context.layer_opacities.remove(&submaster_id);
		ctx.default_context.user_submaster_order.retain(|item| item != &submaster_id);

		// Signal to update everything and return
		self.2.notify_one();
		return was_removed;
	}

	pub fn request_blend(&self) {
		self.2.notify_one();
	}
}

#[async_trait]
impl Savable for MixerInterface {
	async fn save_data(&self) -> Result<Option<Vec<u8>>, String> {
		// return Ok(Some(Vec::new()));
		return Ok(Some(self.1.read().await.serialize_cbor()?));
	}
}

struct SubmasterTypeSpecifier(MixerInterface);

#[async_trait]
impl TypeSpecifier for SubmasterTypeSpecifier {
	async fn get_options(&self) -> Vec<DropdownOptionNative> {
		let submasters = self.0.list_submasters().await;
		let mixer_internals = self.0.1.read().await;

		return submasters
			.into_iter()
			.map(|submaster_id| DropdownOptionNative {
				name: if let Some(submaster) = mixer_internals
					.default_context
					.user_submasters
					.get(&submaster_id)
				{
					submaster.name.clone()
				} else {
					"ERROR: Broken submaster reference".into()
				},
				description: None,
				value: Box::new(submaster_id),
			})
			.collect();
	}
}


#[portable]
#[derive(Debug, Error)]
/// An error that could occur while initializing the mixer plugin
pub enum MixerInitializationError {
	#[error("An error occurred while importing mixer data.")]
	UnrecognizedData,
}

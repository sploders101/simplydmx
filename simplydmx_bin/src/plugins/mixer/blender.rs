use std::time::Instant;
use async_std::{
	channel::{
		self,
		Sender,
	},
	sync::{
		Arc,
		Mutex,
	},
};
use simplydmx_plugin_framework::*;
use crate::type_extensions::uuid::Uuid;
use super::state::{
	MixerContext,
	FullMixerOutput,
	BlendingScheme,
	FullMixerBlendingData,
	BlendingData,
	BlenderValue,
	LayerBin,
	SnapData,
};

#[derive(Eq, PartialEq)]
pub enum UpdateList {

	/// Indicates the patch settings have been changed, so a new base layer and full re-blend is required
	PatcherUpdate,

	/// Indicates a full re-blend is required
	All,

	/// Indicates anything containing the specified submaster should be re-blended
	Submaster(Uuid),

	/// Indicates a re-blend of all cached `LayerBin` outputs is required (ex. Blind transitions)
	LayerBin,

}

/// Start the blending engine
///
/// This function creates a task for
pub async fn start_blender(plugin_context: PluginContext, ctx: Arc<Mutex<MixerContext>>) -> Sender<UpdateList> {
	let (update_sender, update_receiver) = channel::bounded::<UpdateList>(5);

	// Subscribe to patcher updates and notify blending process when they occur
	let plugin_context_patcher_updates = plugin_context.clone();
	let update_sender_patcher = update_sender.clone();
	plugin_context.spawn_volatile(async move {
		let plugin_context = plugin_context_patcher_updates;
		let listener = plugin_context.on::<()>(String::from("patcher.patch_updated")).await;
		loop {
			let event = listener.receive().await;
			match event {
				Event::Msg(_) => update_sender_patcher.send(UpdateList::PatcherUpdate).await.ok(),
				Event::Shutdown => break,
			};
		}
	}).await;

	// Spawn the blender task when its dependencies have been satisfied.
	let plugin_context_blender = plugin_context.clone();
	plugin_context.spawn_when_volatile(vec![
		Dependency::service("patcher", "get_base_layer"),
	], async move {
		let mut patcher_data = ArcAny::<(FullMixerOutput, FullMixerBlendingData)>::new(Arc::new(call_service!(plugin_context_blender, "patcher", "get_base_layer"))).unwrap();
		loop {
			match update_receiver.recv().await {
				Ok(command) => {
					let mut patcher_update = command == UpdateList::PatcherUpdate;
					let mut all_update = command == UpdateList::All;
					let mut reblend_submasters: Vec<Uuid> = if let UpdateList::Submaster(submaster_id) = command {
						vec![submaster_id]
					} else {
						vec![]
					};

					while let Ok(command) = update_receiver.try_recv() {
						match command {
							UpdateList::PatcherUpdate => patcher_update = true,
							UpdateList::All => all_update = true,
							UpdateList::Submaster(submaster_id) => reblend_submasters.push(submaster_id),
							UpdateList::LayerBin => {}, // All commands initiate a LayerBin re-blend
						}
					}

					let mut ctx = ctx.lock().await;

					#[cfg(feature = "blender-benchmark")]
					let start = Instant::now();

					// Patcher updates come first, since they may have data necessary for blending
					if patcher_update {
						patcher_data = ArcAny::new(Arc::new(call_service!(plugin_context_blender, "patcher", "get_base_layer"))).unwrap();
						prune_blender(&mut ctx, &patcher_data).await;
					}

					// Blend any relevant `LayerBin`s based on `all_update` and `reblend_submasters`
					let mut intermediate_bin_cache = Vec::<(Uuid, FullMixerOutput)>::new();
					for layer_bin_id in ctx.layer_bin_order.iter() {
						if let Some(layer_bin_data) = ctx.layer_bins.get(layer_bin_id) {
							// Check relevance of current layer
							let mut found_relevant_layer = all_update;
							if !found_relevant_layer {
								for layer_id in reblend_submasters.iter() {
									if layer_bin_data.layer_order.contains(&layer_id) {
										found_relevant_layer = true;
										break;
									}
								}
							}

							// If relevant, start blending
							if found_relevant_layer {
								let bin_output = blend_bin(&patcher_data, &ctx, &layer_bin_data).await;
								// Save to separate variable to get around active immutable reference
								intermediate_bin_cache.push((layer_bin_id.clone(), bin_output));
							}
						}
					}
					// Update the bin cache now that the immutible reference has been dropped
					for (layer_bin_id, bin_output) in intermediate_bin_cache {
						let bin_arc = Arc::new(bin_output);
						ctx.output_cache.layer_bins.insert(layer_bin_id.clone(), Arc::clone(&bin_arc));
						// Send events now that the cache has been updated
						plugin_context_blender.emit(String::from("mixer.layer_bin_output.") + &layer_bin_id.to_string(), Arc::clone(&bin_arc)).await;
						plugin_context_blender.emit(String::from("mixer.layer_bin_output"), Arc::clone(&bin_arc)).await;
					}

					let mut final_results: FullMixerOutput = patcher_data.0.clone();

					// Blend layer bins together
					// TODO: Make this more efficient. We could open all layer bins at once and loop the attributes.
					for layer_bin_id in ctx.layer_bin_order.iter() {
						if let (
							Some(layer_bin_data),
							Some(layer_bin_opacity),
						) = (
							ctx.output_cache.layer_bins.get(layer_bin_id),
							ctx.layer_bin_opacities.get(layer_bin_id),
						) {
							// All layer_bin blending is LTP static, with potential snapping
							for (fixture_id, fixture_data) in layer_bin_data.iter() {
								if let Some(light_data) = patcher_data.1.get(fixture_id) {
									for (attribute_id, attribute_value) in fixture_data.iter() {
										if let (
											Some(attribute_data),
											Some(fixture_result),
										) = (
											light_data.get(attribute_id),
											final_results.get_mut(fixture_id),
										) {
											if let Some(attribute_result) = fixture_result.get_mut(attribute_id) {
												// attribute_data: used for snapping
												// attribute_value: value to blend
												// fixture_id: used for indexing running results
												// attribute_id: used for indexing running results
												// attribute_result: Current, running value
												// layer_bin_opacity: Opacity of the current layer bin
												match attribute_data.snap {
													SnapData::NoSnap => {
														// Always LTP for layer bin blending
														let faded_value = blend_ltp(
															attribute_result.clone(),
															attribute_value.clone(),
															layer_bin_opacity.clone()
														);
														*attribute_result = faded_value;
													},
													SnapData::SnapAt(snap_threshold) => {
														if layer_bin_opacity > &snap_threshold {
															*attribute_result = attribute_value.clone();
														}
													},
												}
											}
										}
									}
								}
							}
						}
					}

					// Final output is ready
					let final_results = Arc::new(final_results);
					ctx.output_cache.final_output = Arc::clone(&final_results);
					plugin_context_blender.emit(String::from("mixer.final_output"), final_results).await;

					#[cfg(feature = "blender-benchmark")]
					call_service!(plugin_context_blender, "core", "log", format!("Blender took {:?} to run.", start.elapsed()));

				},
				Err(_) => break,
			};
		}
	}).await;

	return update_sender;
}

/// Prunes all submasters of values associated with missing attributes
async fn prune_blender(ctx: &mut MixerContext, patcher_data: &(FullMixerOutput, FullMixerBlendingData)) -> () {
	// Iterate over submasters
	for submaster_data in ctx.submasters.values_mut() {
		// Iterate over fixtures
		let fixture_keys: Vec<Uuid> = submaster_data.data.keys().cloned().collect();
		for fixture_id in fixture_keys {
			if let Some(fixture_base) = patcher_data.0.get(&fixture_id) {
				let fixture_data = submaster_data.data.get_mut(&fixture_id).unwrap(); // unwrapped because key was sourced from here
				// Iterate over attributes
				let attribute_keys: Vec<String> = fixture_data.keys().cloned().collect();
				for attribute_id in attribute_keys {
					if !fixture_base.contains_key(&attribute_id) {
						// Delete attributes that no longer exist
						fixture_data.remove(&attribute_id);
					}
				}
			} else {
				// Delete fixtures that no longer exist
				submaster_data.data.remove(&fixture_id);
			}
		}
	}
}

/// Blend a layer bin together and return the output.
///
/// This function uses the FullMixerOutput from the patcher as the base layer, and blends layers on top of it.
/// It blends all submasters contained within the layer bin into a final result that can be cached in the mixer context.
async fn blend_bin(patcher_data: &(FullMixerOutput, FullMixerBlendingData), ctx: &MixerContext, layer_bin_data: &LayerBin) -> FullMixerOutput {
	let mut cumulative_layer = patcher_data.0.clone();
	// For each submaster in the bin...
	for submaster_id in layer_bin_data.layer_order.iter() {
		if let (
			Some(submaster),
			Some(submaster_opacity),
		) = (
			ctx.submasters.get(submaster_id),
			layer_bin_data.layer_opacities.get(submaster_id),
		) {
			if submaster_opacity == &0u16 {
				continue;
			}
			// For each light within the submaster
			for (fixture_id, fixture_data) in submaster.data.iter() {
				// Get the fixture blending instructions and layer
				if let (
					Some(fixture_blending_data),
					Some(cumulative_fixture),
				) = (
					patcher_data.1.get(fixture_id),
					cumulative_layer.get_mut(fixture_id),
				) {
					// For each property within the light
					for (attribute_id, attribute_value) in fixture_data.iter() {
						// Get blending instructions
						if let (
							Some(blending_data),
							Some(cumulative_attribute),
						) = (
							fixture_blending_data.get(attribute_id),
							cumulative_fixture.get(attribute_id),
						) {
							// Blend the value
							match attribute_value {
								BlenderValue::Offset(attribute_value) => {
									// Blending scheme not used for offset values
									let submaster_opacity = opacity_modifier(submaster_opacity.clone(), blending_data);
									let faded_value = f64::from(attribute_value.clone()) * (f64::from(submaster_opacity) / f64::from(u16::MAX));
									let new_value = f64::from(cumulative_attribute.clone()) + faded_value;
									cumulative_fixture.insert(attribute_id.clone(), new_value.clamp(u16::MIN as f64, u16::MAX as f64).round() as u16);
								},
								BlenderValue::Static(attribute_value) => {
									match blending_data.scheme {
										BlendingScheme::HTP => {
											let submaster_opacity = opacity_modifier(submaster_opacity.clone(), blending_data);
											let faded_value = f64::from(attribute_value.clone()) * (f64::from(submaster_opacity) / f64::from(u16::MAX));
											let faded_value = faded_value.clamp(0f64, 65535f64).round() as u16;
											if &faded_value > cumulative_attribute {
												cumulative_fixture.insert(attribute_id.clone(), faded_value);
											}
										},
										BlendingScheme::LTP => {
											// Avoid simultaneous mutable & immutable borrow by cloning upfront
											let cumulative_attribute = cumulative_attribute.clone();
											// Blend and insert faded value
											cumulative_fixture.insert(
												attribute_id.clone(),
												blend_ltp(
													cumulative_attribute,
													attribute_value.clone(),
													submaster_opacity.clone()
												)
											);
										}
									};
								},
							};
						}
					}
				}
			}
		}
	}
	return cumulative_layer;
}

/// Aids in snapping, so an attribute that uses the "snapping" feature doesn't blend.
///
/// If the opacity is past the snapping threshold, this function returns maximum opacity.
/// If it is below the threshold, it returns 0.
fn opacity_modifier(submaster_opacity: u16, blending_data: &BlendingData) -> u16 {
	return match blending_data.snap {
		SnapData::NoSnap => submaster_opacity,
		SnapData::SnapAt(fulcrum) => if submaster_opacity > fulcrum { u16::MAX } else { u16::MIN },
	};
}

/// Blend two values as LTP, taking opacity into account
fn blend_ltp(current_value: u16, new_value: u16, opacity: u16) -> u16 {
	let start = f64::from(current_value);
	let end = f64::from(new_value);
	let opacity = f64::from(opacity);
	let max_opacity = f64::from(u16::MAX);
	let faded_value: f64 = (end - start) * (opacity / max_opacity) + start;
	let faded_value = faded_value.clamp(0f64, 65535f64).round() as u16;
	return faded_value;
}

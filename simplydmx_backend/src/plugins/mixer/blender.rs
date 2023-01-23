use super::state::{MixerContext, MixingContext};
use async_std::{
	channel::{self, Sender},
	sync::{Arc, RwLock},
	task,
};
use futures::{select, FutureExt};
use simplydmx_plugin_framework::*;
use uuid::Uuid;
use std::time::{Duration, Instant};

use crate::{
	mixer_utils::{
		data_sources::LayerDataSources,
		layer::MixerLayer,
		state::{FullMixerBlendingData, FullMixerOutput},
	},
	plugins::patcher::PatcherInterface,
};

/// Start the blending engine
///
/// This function creates a task for
pub async fn start_blender(
	plugin_context: PluginContext,
	ctx: Arc<RwLock<MixerContext>>,
	patcher_interface: PatcherInterface,
) -> Sender<()> {
	let (sender, receiver) = channel::unbounded();

	// Spawn the blender task when its dependencies have been satisfied.
	let plugin_context_blender = plugin_context.clone();
	plugin_context.spawn_when_volatile("Blender", vec![
		Dependency::flag("saver", "finished"),
	], async move {
		let plugin_context = plugin_context_blender;
		let (base_layer, blending_data): (FullMixerOutput, FullMixerBlendingData) = patcher_interface.get_base_layer().await;
		let data_sources = LayerDataSources {
			base_layer: RwLock::new(Arc::new(base_layer)),
			blending_data: RwLock::new(Arc::new(blending_data)),
		};

		// Set up patch updated listener
		match plugin_context.listen::<()>(String::from("patcher.patch_updated"), FilterCriteria::None).await {
			Ok(listener) => {
				let mut broken_link = false;
				loop {
					let start = Instant::now();

					// Indicates that a layer in the stack is an animation and should hold the blender active
					let mut animated = false;

					// Unlock context
					let ctx_read = ctx.read().await;

					let locked_data_sources = data_sources.lock().await;

					#[cfg(feature = "blender-benchmark")]
					let start_bench = Instant::now();
					let mut cumulative_layer: FullMixerOutput = locked_data_sources.base_layer().clone();
					for layer_id in ctx_read.default_context.layer_order.iter() {
						if let Some(opacity) = ctx_read.default_context.layer_opacities.get(layer_id) {
							if *opacity == 0 { continue } // Skip if opacity is 0
							if let Some(layer) = ctx_read.default_context.user_submasters.get(layer_id) {
								if layer.animated() {
									animated = true;
								}
								layer.blend(&mut cumulative_layer, &locked_data_sources, *opacity).await;
							}
						}
					}
					#[cfg(feature = "blender-benchmark")]
					println!("Blender took {:?}", start_bench.elapsed());

					let result = Arc::new(cumulative_layer);
					patcher_interface.write_values(Arc::clone(&result)).await;

					drop(ctx_read);

					// Rate-limit the blender to cut down on unnecessary CPU usage
					select! {

						// Patcher updates and shutdown requests can interrupt rate-limiting
						msg = listener.receive().fuse() => match msg {
							Event::Msg { .. } => {
								let patcher_data: (FullMixerOutput, FullMixerBlendingData) = patcher_interface.get_base_layer().await;
								ctx.write().await.cleanup(&patcher_data);
								*data_sources.base_layer.write().await = Arc::new(patcher_data.0);
								*data_sources.blending_data.write().await = Arc::new(patcher_data.1);
							},
							Event::Shutdown => break,
						},

						// Rate limiting
						_ = async {
							if animated {
								loop {
									select! {

										// We're running in an animated loop, so keep the queue empty.
										// This allows interrupting the sleep method to discard messages
										_ = receiver.recv().fuse() => {},

										// Wait for the timeout to complete, then break out of the loop
										_ = task::sleep(Duration::from_millis(18).saturating_sub(start.elapsed())).fuse() => break,

									}
								}
							} else {

								// Space updates out at 18+ ms per
								task::sleep(Duration::from_millis(18).saturating_sub(start.elapsed())).await;

								// Wait for an update to come in, and indicate if the update link is broken
								if let Err(_) = receiver.recv().await {
									broken_link = true;
								}

							}
						}.fuse() => {},
					}

					// Broken link indicates we can't receive updates anymore. This is a critical error. The receiving end should drop first in the event of a successful shutdown.
					if broken_link {
						log_error!(plugin_context, "[CRITICAL] SimplyDMX has dropped all references to the mixer's update sender. This is a critical error; please report this to the devs!");
						break;
					}
				}
			},
			Err(error) => {
				log_error!(plugin_context, "[CRITICAL] An error occurred when setting up the blender: {:?}", error);
			},
		}
	}).await;

	return sender;
}

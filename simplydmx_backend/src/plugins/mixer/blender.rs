use super::state::MixerContext;
use simplydmx_plugin_framework::*;
use std::{
	sync::Arc,
	time::{Duration, Instant},
};
use tokio::{
	sync::{
		RwLock,
		Notify,
	},
	time,
	select,
};

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
) -> Arc<Notify> {
	let notifier = Arc::new(Notify::new());
	let notifier_inner = Arc::clone(&notifier);

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
			Ok(mut listener) => {
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
					// TODO: These events don't need to be so frequent
					plugin_context.emit_borrowed("mixer.final_output".into(), FilterCriteria::None, Arc::clone(&result)).await;
					patcher_interface.write_values(Arc::clone(&result)).await;

					drop(ctx_read);

					// Rate-limit the blender to cut down on unnecessary CPU usage
					select! {

						// Patcher updates and shutdown requests can interrupt rate-limiting
						msg = listener.receive() => match msg {
							Event::Msg { .. } => {
								let patcher_data: (FullMixerOutput, FullMixerBlendingData) = patcher_interface.get_base_layer().await;
								ctx.write().await.cleanup(&patcher_data).await;
								*data_sources.base_layer.write().await = Arc::new(patcher_data.0);
								*data_sources.blending_data.write().await = Arc::new(patcher_data.1);
							},
							Event::Shutdown => break,
						},

						// Rate limiting
						_ = async {
							// Space updates out at 18 ms per
							time::sleep(Duration::from_millis(18).saturating_sub(start.elapsed())).await;

							// Wait for an update to come in if no layers are animated
							if !animated {
								notifier_inner.notified().await;
							}
						} => {},
					}
				}
			},
			Err(error) => {
				log_error!(plugin_context, "[CRITICAL] An error occurred when setting up the blender: {:?}", error);
			},
		}
	}).await;

	return notifier;
}

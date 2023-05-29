use sacn::DmxSource;
use simplydmx_plugin_framework::{PluginContext, Dependency};
use std::{
	collections::{HashMap, HashSet},
	sync::Arc,
	time::{Duration, Instant},
};
use tokio::{sync::Mutex, time};
use uuid::uuid;

pub type ControllerCache = Arc<Mutex<Option<HashMap<u16, [u8; 512]>>>>;

// TODO: Refactor sacn with async I/O. It should be fast enough though not to cause any noticable issue in the meantime
//       This task should also shut down or sit idle when not needed. Currently, the loop still runs without data.
pub async fn initialize_controller(plugin_context: PluginContext) -> ControllerCache {
	let cache = Arc::new(Mutex::new(Some(HashMap::<u16, [u8; 512]>::new())));
	let e131_cache = Arc::clone(&cache);

	let mut shutdown_listener = plugin_context.on_shutdown().await;
	plugin_context.spawn_when_volatile("E.131 DMXSource Controller", vec![
		Dependency::flag("saver", "finished"),
	], async move {
		let mut controller: Option<DmxSource> = None;
		let mut previous_universes = HashSet::<u16>::new();

		loop {
			if let Ok(()) = shutdown_listener.try_recv() {
				if let Some(controller) = controller {
					for universe_id in previous_universes {
						controller.terminate_stream(universe_id).ok();
					}
				}
				break;
			}
			let unlocked_cache = e131_cache.lock().await;
			let mut current_universes = HashSet::<u16>::new();
			let started_loop = Instant::now();

			if let Some(ref unlocked_cache) = *unlocked_cache {
				if unlocked_cache.len() > 0 && controller.is_none() {
					controller = DmxSource::with_cid("SimplyDMX", uuid!("7725e1df-53cf-4365-b530-ce1705ca1860")).ok();
					if controller.is_none() {
						drop(unlocked_cache);
						time::sleep(Duration::from_secs(5)).await;
						continue;
					}
				} else if unlocked_cache.len() == 0 && controller.is_some() {
					controller = None;
				}

				if let Some(ref controller) = controller {
					for (universe_id, dmx_frame) in unlocked_cache.iter() {
						previous_universes.remove(&universe_id);
						current_universes.insert(*universe_id);
						controller.send(*universe_id, dmx_frame).ok();
					}
					for universe_id in previous_universes {
						controller.terminate_stream(universe_id).ok();
					}
					previous_universes = current_universes;
				}
			}
			drop(unlocked_cache);

			let sleep_duration = Duration::from_millis(18).saturating_sub(started_loop.elapsed());
			if !sleep_duration.is_zero() {
				time::sleep(sleep_duration).await;
			}
		}
	}).await;

	return cache;
}

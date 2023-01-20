use async_std::{
	channel,
	sync::{Arc, Mutex},
	task::block_on,
};
use std::{
	collections::{HashMap, HashSet},
	sync::atomic::{AtomicBool, Ordering},
	thread,
	time::{Duration, Instant},
};

use sacn::DmxSource;
use simplydmx_plugin_framework::PluginContext;

pub type ControllerCache = Arc<Mutex<Option<HashMap<u16, [u8; 512]>>>>;

pub async fn initialize_controller(plugin_context: PluginContext) -> ControllerCache {
	let cache = Arc::new(Mutex::new(Some(HashMap::<u16, [u8; 512]>::new())));
	let e131_cache = Arc::clone(&cache);

	let (shutdown_confirm_sender, shutdown_confirm_receiver) = channel::bounded::<()>(1);
	let shutting_down: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
	let shutting_down_loader = Arc::clone(&shutting_down);

	thread::spawn(move || {
		let mut controller: Option<DmxSource> = None;
		let mut previous_universes = HashSet::<u16>::new();

		loop {
			if shutting_down_loader.load(Ordering::Relaxed) {
				if let Some(controller) = controller {
					for universe_id in previous_universes {
						controller.terminate_stream(universe_id).ok();
					}
				}
				break;
			}
			let unlocked_cache = block_on(e131_cache.lock());
			let mut current_universes = HashSet::<u16>::new();
			let started_loop = Instant::now();

			if let Some(ref unlocked_cache) = *unlocked_cache {
				if unlocked_cache.len() > 0 && controller.is_none() {
					controller = DmxSource::new("SimplyDMX").ok();
					if controller.is_none() {
						thread::sleep(Duration::from_secs(5));
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

			let sleep_duration = Duration::from_millis(18).saturating_sub(started_loop.elapsed());
			if !sleep_duration.is_zero() {
				thread::sleep(sleep_duration);
			}
		}

		block_on(shutdown_confirm_sender.send(())).ok();
	});

	plugin_context
		.register_finisher("E.131 Shutdown", async move {
			shutting_down.store(true, Ordering::Relaxed); // Mark controller thread for shutdown on next iteration
			shutdown_confirm_receiver.recv().await.ok(); // Wait for controller thread to shut down
		})
		.await
		.unwrap();

	return cache;
}

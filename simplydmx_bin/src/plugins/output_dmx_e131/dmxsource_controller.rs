use std::{
	thread,
	collections::{
		HashMap,
		HashSet,
	},
	time::{
		Duration,
		Instant,
	},
};
use async_std::{
	task::block_on,
	sync::{
		Arc,
		Mutex,
	},
};

use sacn::DmxSource;

pub fn initialize_controller() -> Arc<Mutex<Option<HashMap<u16, [u8; 512]>>>> {
	let cache = Arc::new(Mutex::new(Some(HashMap::<u16, [u8; 512]>::new())));
	let e131_cache = Arc::clone(&cache);

	thread::spawn(move || {
		let mut controller: Option<DmxSource> = None;
		let mut previous_universes = HashSet::<u16>::new();

		loop {
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

			let sleep_duration = Duration::from_millis(20).saturating_sub(started_loop.elapsed());
			if !sleep_duration.is_zero() {
				thread::sleep(sleep_duration);
			}
		}
	});

	return cache;
}

use enttecopendmx::EnttecOpenDMX;
use std::{
	sync::{atomic::AtomicBool, Arc},
	time::{Duration, Instant},
};
use thread_priority::{set_current_thread_priority, ThreadPriority};
use tokio::{sync::Mutex, task::JoinHandle};

pub struct OpenDMXController {
	shutdown_trigger: Arc<AtomicBool>,
	channels: Arc<Mutex<Option<[u8; 512]>>>,
	thread_handle: Option<JoinHandle<()>>,
}

impl OpenDMXController {
	pub fn new(initial_channels: [u8; 512]) -> Self {
		let shutdown_trigger = Arc::new(AtomicBool::new(false));
		let thread_shutdown_trigger = Arc::clone(&shutdown_trigger);
		let channels = Arc::new(Mutex::new(Some(initial_channels)));
		let thread_channels = Arc::clone(&channels);
		let thread_handle = tokio::task::spawn_blocking(move || thread_loop(thread_shutdown_trigger, thread_channels));
		return OpenDMXController {
			shutdown_trigger,
			channels,
			thread_handle: Some(thread_handle),
		};
	}

	/// Sends a frame to the DMX controller running in another thread.
	///
	/// NOTE: The DMX controller runs on its own thread with its own
	/// ticking, so this frame is not guaranteed to be sent.
	pub async fn send_frame(&self, channels: [u8; 512]) {
		let mut inner_channels = self.channels.lock().await;
		*inner_channels = Some(channels);
	}

	/// Safely shuts down the controller in a non-blocking way.
	pub async fn shutdown(mut self) {
		self.shutdown_trigger.store(true, std::sync::atomic::Ordering::Relaxed);
		// Panics are show-stoppers and should be propogated
		if let Some(thread_handle) = self.thread_handle.take() {
			thread_handle.await.unwrap();
		} else {
			#[cfg(debug_assertions)]
			eprintln!("OpenDMXController thread handle was missing! This shouldn't be possible.");
		}
	}
}

impl Drop for OpenDMXController {
	fn drop(&mut self) {
		if !self.shutdown_trigger.load(std::sync::atomic::Ordering::Relaxed) {
			#[cfg(debug_assertions)]
			eprintln!("OpenDMXController was inappropriately dropped!");
			self.shutdown_trigger.store(true, std::sync::atomic::Ordering::Relaxed);
		}
	}
}

fn thread_loop(shutdown_trigger: Arc<AtomicBool>, channels: Arc<Mutex<Option<[u8; 512]>>>) {
	set_current_thread_priority(ThreadPriority::Max).ok();
	let mut last_retry = Instant::now();
	let mut port = EnttecOpenDMX::new().and_then(|mut port| {
		port.open()?;
		Ok(port)
	});

	loop {
		if shutdown_trigger.load(std::sync::atomic::Ordering::Relaxed) {
			break;
		}

		match port {
			Ok(ref mut inner_port) => {
				// Receive new values if any are available
				let mut channels_inner = channels.blocking_lock();
				if let Some(ref channels) = *channels_inner {
					for i in 0..512 {
						inner_port.set_channel(i+1, channels[i]);
					}
					*channels_inner = None;
				}
				drop(channels_inner);

				// Render universe
				if let Err(err) = inner_port.render() {
					inner_port.close().ok();
					port = Err(err);
				}
				std::thread::sleep(Duration::from_millis(2));
			},
			Err(_) => {
				// Keep sleep durations small so we can check if we need to shut down frequently.
				// Other tasks may be waiting on us to quit before they can continue.
				while last_retry.elapsed() < Duration::from_secs(2) {
					std::thread::sleep(Duration::from_millis(10));
					if shutdown_trigger.load(std::sync::atomic::Ordering::Relaxed) {
						break;
					}
				}
				last_retry = Instant::now();
				port = EnttecOpenDMX::new().and_then(|mut port| {
					port.open()?;
					Ok(port)
				});
			},
		}
	}
}

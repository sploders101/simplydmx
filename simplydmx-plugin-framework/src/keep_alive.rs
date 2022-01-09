use std::{
	future::Future,
	collections::HashMap,
	pin::Pin,
	sync::Arc,
};

use async_std::{
	task::{
		self,
		JoinHandle,
	},
	sync::{
		RwLock,
		Mutex,
	},
};
use uuid::Uuid;


/// An error returned from a `KeepAlive` registration call
pub enum KeepAliveRegistrationError {
	ShuttingDown,
}

/// An error returned from a `KeepAlive` de-registration call
pub enum KeepAliveDeregistrationError {
	ShuttingDown,
	NotRegistered,
}


/// This struct provides hooks into the application lifecycle and allows preventing
/// the application from exiting before critical tasks are completed.
#[derive(Clone)]
pub struct KeepAlive {
	/// Internal data is held in an arc so references can be easily passed around to
	/// functions running in parallel.
	internal_data: Arc<KeepAliveInternal>,
}


struct KeepAliveInternal {

	/// Indicates that SimplyDMX is shutting down and should not accept more blockers or
	/// finishers.
	shutting_down: RwLock<bool>,

	/// Blockers are run immediately and prevent the application from closing until
	/// they finish.
	blockers: Mutex<HashMap<Uuid, JoinHandle<()>>>,

	/// Finishers are run when the application is about to exit, and prevent the
	/// application from closing until they finish.
	finishers: Mutex<HashMap<Uuid, Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,

}

impl KeepAlive {

	/// Register a routine that should be run to completion before shutting down the application.
	/// Anything that should not be interrupted should be run through this function. If the
	/// application is already shutting down, any call to this function will fail with
	/// `KeepAliveRegistrationError::ShuttingDown`.
	pub async fn register_blocker<F>(&self, blocker: F) -> Result<(), KeepAliveRegistrationError>
	where
		F: Future<Output = ()> + Send + 'static,
	{
		if *self.internal_data.shutting_down.read().await {
			return Err(KeepAliveRegistrationError::ShuttingDown);
		} else {
			let uuid = Uuid::new_v4();

			// Clone internal data so it can be used in an async function
			let internal_data = Arc::clone(&self.internal_data);
			let future = task::spawn(async move {
				blocker.await;
				if !*internal_data.shutting_down.read().await {
					internal_data.blockers.lock().await.remove(&uuid);
				}
			});
			self.internal_data.blockers.lock().await.insert(Uuid::clone(&uuid), future);
			return Ok(());
		}
	}

	/// Registers a finisher function and returns a UUID representing it.
	/// Finisher functions are run before application exit, and allow for
	/// things like saving data quickly before exiting.
	pub async fn register_finisher<F>(&mut self, finisher: F) -> Result<Uuid, KeepAliveRegistrationError>
	where
		F: Future<Output = ()> + Send + 'static,
	{
		if *self.internal_data.shutting_down.read().await {
			return Err(KeepAliveRegistrationError::ShuttingDown);
		} else {
			let uuid = Uuid::new_v4();
			self.internal_data.finishers.lock().await.insert(Uuid::clone(&uuid), Box::pin(finisher));
			return Ok(uuid);
		}
	}

	/// De-registers a finisher function, removing it from the list of tasks
	/// to accomplish before application exit.
	pub async fn deregister_finisher(&mut self, handle: Uuid) -> Result<(), KeepAliveDeregistrationError> {
		if *self.internal_data.shutting_down.read().await {
			return Err(KeepAliveDeregistrationError::ShuttingDown);
		} else {
			self.internal_data.finishers.lock().await.remove(&handle);
			return Ok(());
		}
	}

}

/// Create a new instance of KeepAlive. Plugins should not have the ability to do this, so it
/// is not included as part of the type.
pub fn create_keep_alive() -> KeepAlive {
	return KeepAlive {
		internal_data: Arc::new(KeepAliveInternal {
			shutting_down: RwLock::new(false),
			blockers: Mutex::new(HashMap::new()),
			finishers: Mutex::new(HashMap::new()),
		})
	};
}

/// Initiate the shutdown sequence contained in the KeepAlive. This gives plugins an
/// opportunity to finish what they were doing and perform cleanup routines before quitting
/// the application
pub async fn shut_down(keep_alive: KeepAlive) {

	// Mark that we're shutting down
	*keep_alive.internal_data.shutting_down.write().await = true;

	// Wait for all blockers to finish
	for handle in keep_alive.internal_data.blockers.lock().await.values_mut() {
		// This await does not drive the future, since it was created
		// using task::spawn(...)
		handle.await;
	}

	// Run all finishers in parallel
	let mut finisher_futures = keep_alive.internal_data.finishers.lock().await;
	let finisher_keys = finisher_futures.keys().cloned().collect::<Vec<Uuid>>();
	let finisher_tasks = finisher_keys.into_iter().map(move |finisher_key| {
		let finisher = finisher_futures.remove(&finisher_key).expect("`finishers` was modified while locked!");
		return task::spawn(finisher);
	});
	// Wait for all finishers to complete
	for handle in finisher_tasks {
		handle.await;
	}

}

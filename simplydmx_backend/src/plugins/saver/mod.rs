use std::{collections::HashMap, sync::Arc};

use async_std::sync::RwLock;
use async_trait::async_trait;
pub use simplydmx_plugin_framework::*;

use self::types::ShowFile;

mod services;
mod types;

/// The trait that plugins should implement in order to save show data
#[async_trait]
pub trait Savable: Send + Sync + 'static {
	async fn save_data(&self) -> Result<Option<Vec<u8>>, String>;
}

/// Initialize the saver plugin, returning its interface.
///
/// The saver plugin should be the first plugin initialized, as all other plugins can use its API to consume previously-loaded data.
pub async fn initialize(
	plugin_context: PluginContext,
	loaded_data: Option<Vec<u8>>,
) -> Result<SaverInterface, String> {
	plugin_context.declare_event::<SaverInitializationStatus>(
		"saver.load_status".into(),
		Some("Emitted whenever initialization has finished and indicates if SimplyDMX's state is safe or not.".into()),
	).await.unwrap();

	let loaded_data = if let Some(loaded_data) = loaded_data {
		match ciborium::de::from_reader::<'_, ShowFile, &[u8]>(&loaded_data) {
			Ok(result) => Some(result),
			Err(err) => return Err(format!("{:?}", err)),
		}
	} else {
		None
	};

	let saver_interface = SaverInterface(
		plugin_context.clone(),
		Arc::new(RwLock::new(SaverData {
			status: SaverInitializationStatus::Initializing,
			loaded_data,
			savers: HashMap::new(),
		})),
	);

	plugin_context
		.register_service(true, services::SaveShow::new(saver_interface.clone()))
		.await
		.unwrap();

	return Ok(saver_interface);
}

/// Internal data held and maintained by the saver plugin
pub struct SaverData {
	status: SaverInitializationStatus,
	loaded_data: Option<ShowFile>,
	savers: HashMap<String, Box<dyn Savable>>,
}

/// The saver plugin's interface, used by other plugins to save and load data
#[derive(Clone)]
pub struct SaverInterface(PluginContext, Arc<RwLock<SaverData>>);

impl SaverInterface {
	/// Registers a `Savable<T>` interface
	pub async fn register_savable(
		&self,
		id: impl Into<String>,
		interface: impl Savable,
	) -> Result<(), RegisterSavableError> {
		let id = id.into();
		let mut ctx = self.1.write().await;

		if ctx.savers.contains_key(&id) {
			return Err(RegisterSavableError::SaverAlreadyExists);
		}
		ctx.savers.insert(id, Box::new(interface));
		return Ok(());
	}

	/// Save the show
	pub async fn save_show(&self) -> Result<Vec<u8>, SaveError> {
		let ctx = self.1.read().await;

		let mut show_file = ShowFile {
			plugin_data: HashMap::new(),
		};

		for (id, interface) in ctx.savers.iter() {
			match interface.save_data().await {
				Ok(saved_data) => {
					if let Some(saved_data) = saved_data {
						show_file.plugin_data.insert(String::clone(id), saved_data);
					}
				}
				Err(error) => return Err(SaveError::SaverReturnedErr { error }),
			}
		}

		let mut serialized = Vec::<u8>::new();
		ciborium::ser::into_writer(&show_file, &mut serialized)?;
		return Ok(serialized);
	}

	/// Obtains a plugin's data from a file
	pub async fn load_data<T: BidirectionalPortable>(
		&self,
		id: &String,
	) -> Result<Option<T>, String> {
		let mut ctx = self.1.write().await;
		if let Some(ref mut loaded_data) = ctx.loaded_data {
			if let Some(encoded_data) = loaded_data.plugin_data.remove(id) {
				return match ciborium::de::from_reader::<'_, T, &[u8]>(&encoded_data) {
					Ok(result) => Ok(Some(result)),
					Err(error) => Err(format!("{:?}", error)),
				};
			}
		}
		return Ok(None);
	}

	pub async fn finish_initialization(&self) -> SaverInitializationStatus {
		let mut ctx = self.1.write().await;

		if let Some(ref loaded_data) = ctx.loaded_data {
			if loaded_data.plugin_data.len() > 0 {
				ctx.status = SaverInitializationStatus::FinishedUnsafe;
			} else {
				ctx.status = SaverInitializationStatus::FinishedSafe;
			}
		} else {
			ctx.status = SaverInitializationStatus::FinishedSafe;
		}

		self.0
			.emit(
				"saver.load_status".into(),
				FilterCriteria::None,
				ctx.status.clone(),
			)
			.await;
		self.0.set_init_flag("finished".into()).await;

		return ctx.status.clone();
	}

	pub async fn get_status(&self) -> SaverInitializationStatus {
		return self.1.read().await.status.clone();
	}
}

/// An error returned when registering a saver. This is usually okay to unwrap, since it should be during init
#[portable]
#[serde(tag = "type")]
pub enum RegisterSavableError {
	SaverAlreadyExists,
}

/// An error returned by the saver if saving data failed
#[portable]
#[serde(tag = "type", content = "data")]
pub enum SaveError {
	SaverReturnedErr {
		error: String,
	},
	ErrorSerializing {
		error: String,
	},
	/// This error is returned when a save operation would be considered unsafe, such as halfway through initialization
	/// or if any unrecognized data is in the file.
	Unsafe,
}
impl<T: std::fmt::Debug> From<ciborium::ser::Error<T>> for SaveError {
	fn from(error: ciborium::ser::Error<T>) -> Self {
		match error {
			ciborium::ser::Error::Io(error) => Self::ErrorSerializing {
				error: format!("{:?}", error),
			},
			ciborium::ser::Error::Value(error) => Self::ErrorSerializing { error },
		}
	}
}

#[portable]
#[serde(tag = "type")]
/// Describes the state of the show controller backend during initialization
pub enum SaverInitializationStatus {
	FinishedSafe,
	FinishedUnsafe,
	Initializing,
}

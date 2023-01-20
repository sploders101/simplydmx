use super::{SaveError, SaverInterface};
use simplydmx_plugin_framework::*;

#[interpolate_service("save", "Save Show", "Saves the show, returning the raw byte vector")]
impl SaveShow {
	#![inner_raw(SaverInterface)]

	pub fn new(saver_interface: SaverInterface) -> Self {
		Self(saver_interface)
	}

	#[service_main(
		("Show File", "The raw byte output to be saved to a file"),
	)]
	async fn save(self) -> Result<Vec<u8>, SaveError> {
		return self.0.save_show().await;
	}
}

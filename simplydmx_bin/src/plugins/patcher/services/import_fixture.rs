use async_std::sync::{
	Arc,
	RwLock,
};

use crate::plugins::patcher::fixture_types::SerializedData;

use super::super::fixture_types::FixtureBundle;
use super::super::state::PatcherContext;
use simplydmx_plugin_framework::*;

#[interpolate_service(
	"import_fixture",
	"Import Fixture",
	"Imports a fixture in the form of a FixtureBundle object",
)]
impl ImportFixture {

	#![inner_raw(PluginContext, Arc::<RwLock::<PatcherContext>>)]

	#[service_main(
		("Fixture Bundle", "Post-deserialized fixture bundle"),
		("Result", "The result of the import. The error will be in human-readable format."),
	)]
	pub async fn main(self, fixture_bundle: FixtureBundle) -> Result<(), String> {


		return Ok(());
	}

}

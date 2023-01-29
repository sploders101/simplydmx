pub mod driver_types;
pub mod fixture_types;
pub mod interface;
pub mod services;
pub mod state;

use async_std::task;
use async_trait::async_trait;
use simplydmx_plugin_framework::*;

use self::interface::DMXInterface;

use super::{patcher::PatcherInterface, saver::SaverInterface, mixer::MixerInterface};

pub async fn initialize(
	plugin_context: PluginContext,
	saver: SaverInterface,
	mixer_interface: MixerInterface,
	patcher_interface: PatcherInterface,
) -> Result<DMXInterface, DMXInitializationError> {
	// Create plugin interface
	let output_context = if let Ok(data) = saver.load_data(&"output_dmx".into()).await {
		if let Some(data) = data {
			DMXInterface::from_file(plugin_context.clone(), data)
		} else {
			DMXInterface::new(plugin_context.clone())
		}
	} else {
		return Err(DMXInitializationError::UnrecognizedData);
	};

	patcher_interface
		.register_output_driver(output_context.clone())
		.await;

	plugin_context
		.declare_event::<Vec<u8>>(
			"dmx.output".into(),
			Some("The output of the DMX plugin, for display by the UI. This should not be used by DMX drivers.".into()),
		)
		.await
		.unwrap();

	plugin_context
		.declare_event::<()>(
			"dmx.universe_linked".into(),
			Some("Emitted whenever a universe is linked to a transport driver.".into()),
		)
		.await
		.unwrap();
	plugin_context
		.declare_event::<()>(
			"dmx.universe_unlinked".into(),
			Some("Emitted whenever a universe is unlinked from a transport driver.".into()),
		)
		.await
		.unwrap();
	plugin_context
		.declare_event::<()>(
			"dmx.universes_updated".into(),
			Some("Emitted whenever a universe is created or deleted.".into()),
		)
		.await
		.unwrap();
	plugin_context
		.declare_event::<()>(
			"dmx.drivers_updated".into(),
			Some("Emitted whenever a driver is added.".into()),
		)
		.await
		.unwrap();

	plugin_context
		.declare_event::<()>(
			"dmx.universes_changed".into(),
			Some("Emitted when a universe is removed from the DMX plugin.".into()),
		)
		.await
		.unwrap();

	plugin_context
		.register_service(true, services::CreateUniverse::new(output_context.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, services::DeleteUniverse::new(output_context.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, services::LinkUniverse::new(output_context.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, services::UnlinkUniverse::new(output_context.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, services::ListUniverses::new(output_context.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, services::ListDrivers::new(output_context.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, services::GetLinkedController::new(output_context.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, services::GetLinkUniverseForm::new(output_context.clone()))
		.await
		.unwrap();
	plugin_context
		.register_service(true, services::RenameUniverse::new(output_context.clone()))
		.await
		.unwrap();

	plugin_context
		.register_service_type_specifier(
			"universes".into(),
			UniverseTypeSpecifier(output_context.clone()),
		)
		.await
		.unwrap();
	plugin_context
		.register_service_type_specifier(
			"universes_optional".into(),
			OptionalUniverseTypeSpecifier(output_context.clone()),
		)
		.await
		.unwrap();
	plugin_context
		.register_service_type_specifier(
			"dmx_drivers_optional".into(),
			OptionalDMXDriverTypeSpecifier(output_context.clone()),
		)
		.await
		.unwrap();

	saver
		.register_savable("output_dmx", output_context.clone())
		.await
		.unwrap();

	let mixer_interface_universe_linked = mixer_interface.clone();
	plugin_context.on::<()>("dmx.universe_linked", FilterCriteria::None, move |_, _| {
		let mixer_interface = mixer_interface_universe_linked.clone();
		println!("Starting requester task");
		task::spawn(async move {
			println!("Requesting blend");
			mixer_interface.request_blend().await;
		});
	}).await.unwrap().drop();

	return Ok(output_context);
}

#[portable]
/// An error that could occur while initializing the DMX plugin
pub enum DMXInitializationError {
	UnrecognizedData,
}

pub struct UniverseTypeSpecifier(DMXInterface);
#[async_trait]
impl TypeSpecifier for UniverseTypeSpecifier {
	async fn get_options(&self) -> Vec<DropdownOptionNative> {
		return self
			.0
			.list_universes()
			.await
			.into_iter()
			.map(|(id, name)| DropdownOptionNative {
				name,
				description: None,
				value: Box::new(id),
			})
			.collect();
	}
}

pub struct OptionalUniverseTypeSpecifier(DMXInterface);
#[async_trait]
impl TypeSpecifier for OptionalUniverseTypeSpecifier {
	async fn get_options(&self) -> Vec<DropdownOptionNative> {
		let mut options = vec![DropdownOptionNative {
			name: String::from("Unassigned"),
			description: None,
			value: Box::new(Option::<uuid::Uuid>::None),
		}];
		options.extend(self.0.list_universes().await.into_iter().map(|(id, name)| {
			DropdownOptionNative {
				name,
				description: None,
				value: Box::new(Some(id)),
			}
		}));
		return options;
	}
}

pub struct OptionalDMXDriverTypeSpecifier(DMXInterface);
#[async_trait]
impl TypeSpecifier for OptionalDMXDriverTypeSpecifier {
	async fn get_options(&self) -> Vec<DropdownOptionNative> {
		let mut options = vec![DropdownOptionNative {
			name: String::from("Unlinked"),
			description: None,
			value: Box::new(Option::<uuid::Uuid>::None),
		}];
		options.extend(self.0.list_drivers().await.into_iter().map(|description| {
			DropdownOptionNative {
				name: description.name,
				description: Some(description.description),
				value: Box::new(Some(description.id)),
			}
		}));
		return options;
	}
}

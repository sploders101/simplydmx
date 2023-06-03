use crate::plugins::{self, saver::SaverInitializationStatus};
use simplydmx_plugin_framework::PluginManager;

// Public so the GUI plugin can run it
pub async fn async_main(plugin_manager: &PluginManager, data: Option<Vec<u8>>) {
	// TODO: Error handling during init. This wasn't originally intended to be necessary,
	// but since file loading re-starts plugins with untrusted data, it needs to be done.

	let saver = plugins::saver::initialize(
		plugin_manager
			.register_plugin("saver", "Data Saver/Loader")
			.await
			.unwrap(),
		data,
	)
	.await
	.unwrap();

	// Register core plugin
	plugins::core::initialize(
		plugin_manager
			.register_plugin("core", "SimplyDMX Core")
			.await
			.unwrap(),
	)
	.await;

	let patcher_interface = plugins::patcher::initialize(
		plugin_manager
			.register_plugin("patcher", "SimplyDMX Fixture Patcher")
			.await
			.unwrap(),
		saver.clone(),
	)
	.await
	.unwrap();

	let mixer_interface = plugins::mixer::initialize_mixer(
		plugin_manager
			.register_plugin("mixer", "SimplyDMX Mixer")
			.await
			.unwrap(),
		saver.clone(),
		patcher_interface.clone(),
	)
	.await
	.unwrap();

	#[cfg(feature = "output-dmx")]
	let dmx_interface = plugins::output_dmx::initialize(
		plugin_manager
			.register_plugin("output_dmx", "DMX Universe Renderer")
			.await
			.unwrap(),
		saver.clone(),
		mixer_interface.clone(),
		patcher_interface.clone(),
	)
	.await
	.unwrap();

	#[cfg(feature = "output-dmx-e131")]
	plugins::output_dmx_e131::initialize(
		plugin_manager
			.register_plugin("output_dmx_e131", "E1.31/sACN DMX Output")
			.await
			.unwrap(),
		saver.clone(),
		dmx_interface.clone(),
	)
	.await
	.unwrap();

	#[cfg(feature = "output-dmx-enttecopendmx")]
	plugins::output_dmx_enttecopendmx::initialize(
		plugin_manager
			.register_plugin("output_dmx_enttecopendmx", "Enttec OpenDMX Output")
			.await
			.unwrap(),
		saver.clone(),
		dmx_interface.clone(),
	)
	.await
	.unwrap();

	let init_status = saver.finish_initialization().await;
	if let SaverInitializationStatus::FinishedUnsafe = init_status {
		panic!(
			"Save file contains features that are not compatible with this version of SimplyDMX"
		);
	}
}

#[cfg(feature = "export-services")]
pub mod exporter {
	use linkme::distributed_slice;
	use simplydmx_plugin_framework::{
		DropdownOptionJSON, FilterCriteria, PluginManager, ServiceArgumentOwned,
		ServiceDescription, TypeSpecifierRetrievalError,
	};
	use tokio::runtime::Runtime;
	use std::{cmp::Ordering, collections::HashMap, fs::File, io::Write};
	use tsify::Tsify;

	#[distributed_slice]
	pub static PORTABLETYPE: [(&'static str, &'static str, &'static Option<&'static str>)] = [..];

	pub fn rpc_coverage() {
		let runtime = Runtime::new().expect("Couldn't start tokio runtime");
		let _runtime_guard = runtime.enter();

		// Create plugin manager
		let plugin_manager = PluginManager::new();

		// Initialize plugins
		runtime.block_on(super::async_main(&plugin_manager, None));

		// Sort types to enable deterministic exports for git tracking
		let mut types: Vec<(&'static str, &'static str, &'static Option<&'static str>)> = vec![
			(
				"FxHashMap",
				"export type FxHashMap<K extends string | number | symbol, V> = Record<K, V>;",
				&Some(
					r#"/** This is the same as a HashMap, but uses a more efficient hashing algorithm in the backend */"#
				),
			),
			(
				"Uuid",
				"export type Uuid = string;",
				&Some(
					r#"/** Unique identifier used in various parts of the API. In TS, UUID does not have its own data type, so this just re-exports string. */"#,
				),
			),
			(
				"Value",
				"export type Value = any;",
				&Some(
					r#"/** Represents Rust's `serde_json::Value` type. This is used for dynamic typing, like when using backend-defined forms. */"#,
				),
			),
			(
				"FilterCriteria",
				FilterCriteria::DECL,
				&Some(
					r#"/** Represents criteria used to filter an event. For example, a submaster UUID could be used to filter submaster updates by that specific submaster */"#,
				),
			),
			(
				"ServiceDescription",
				ServiceDescription::DECL,
				&Some(r#"/** Describes a service that can be called from an external API */"#),
			),
			(
				"ServiceArgumentOwned",
				ServiceArgumentOwned::DECL,
				&Some(r#"/** Describes an argument that must be passed to a service call */"#),
			),
			(
				"DropdownOptionJSON",
				DropdownOptionJSON::DECL,
				&Some(r#"/** Describes a value to be shown in a dropdown list */"#),
			),
			(
				"TypeSpecifierRetrievalError",
				TypeSpecifierRetrievalError::DECL,
				&Some(
					r#"/** Describes an error that occurred while retrieving items for a dropdown list */"#,
				),
			),
		];
		types.append(&mut PORTABLETYPE.into_iter().cloned().collect::<Vec<(
			&'static str,
			&'static str,
			&'static Option<&'static str>,
		)>>());
		types.sort_by(|a, b| {
			if a.0 > b.0 {
				return Ordering::Greater;
			} else if a.0 < b.0 {
				return Ordering::Less;
			} else {
				return Ordering::Equal;
			}
		});

		// Export services
		let mut value = runtime.block_on(plugin_manager.list_services());
		value.sort_by(|a, b| {
			if a.plugin_id > b.plugin_id {
				return Ordering::Greater;
			} else if a.plugin_id < b.plugin_id {
				return Ordering::Less;
			} else if a.id > b.id {
				return Ordering::Greater;
			} else if a.id < b.id {
				return Ordering::Less;
			} else {
				return Ordering::Equal;
			}
		});

		let mut plugin_services = HashMap::<String, Vec<ServiceDescription>>::new();
		for service in value {
			if let Some(plugin_services_vec) = plugin_services.get_mut(&service.plugin_id) {
				plugin_services_vec.push(service);
			} else {
				plugin_services.insert(service.plugin_id.clone(), vec![service]);
			}
		}

		let mut rpc_modules = String::new();

		let mut sorted_plugin_ids = plugin_services.keys().cloned().collect::<Vec<String>>();
		sorted_plugin_ids.sort();
		for plugin_id in sorted_plugin_ids {
			let mut services = plugin_services.remove(&plugin_id).unwrap();
			rpc_modules += &format!("\nexport const {} = {{\n", &plugin_id);

			services.sort_by(|a, b| a.id.cmp(&b.id));
			for service in services {
				let mut service_args_with_types = Vec::<String>::new();
				let mut service_args_no_types = Vec::<String>::new();

				for arg in service.arguments {
					// Using [31..] here to trim out Tsify's `type FunctionArgument = `
					// Ideally this would be done in the macro, but there are some weird issues compiling `FunctionArgument::DECL[..]`
					service_args_with_types.push(format!(
						"{}: {}",
						arg.id,
						&arg.val_type[31..arg.val_type.len() - 1]
					));
					service_args_no_types.push(String::from(arg.id));
				}

				rpc_modules += &format!(
					"\t/** {} */\n\t{}({}): Promise<{}> {{ return callService(\"{}\", \"{}\", [{}]) }},\n",
					&service.description,
					&service.id,
					&service_args_with_types.join(", "),
					if let Some(ref arg) = service.returns {
						&arg.val_type[31..arg.val_type.len() - 1]
					} else {
						"void"
					},
					&plugin_id,
					&service.id,
					&service_args_no_types.join(", "),
				);
			}
			rpc_modules += "};\n";
		}

		let manifest_directory =
			std::env::var("CARGO_MANIFEST_DIR").expect("Could not get project directory");
		let mut rpc_path = std::path::PathBuf::from(manifest_directory);
		rpc_path.pop();
		rpc_path.extend([
			"simplydmx_frontend",
			"src",
			"scripts",
			"api",
			"ipc",
			"rpc.ts",
		]);
		let mut rpc_ts = File::create(&rpc_path).unwrap();
		rpc_ts.write_all(format!("// This file is automatically generated by the SimplyDMX plugin framework.\n// Please do not edit it manually.\n\nimport {{ callService }} from \"./agnostic_abstractions\";\n\n\n{}\n\n{}\n", &types.into_iter().map(|ty| {
			if let Some(docs) = ty.2 {
				format!("{}\n{}", docs, ty.1)
			} else {
				format!("/** This type is currently undocumented. I will be working to resolve this for all types in the near future. */\n{}", ty.1)
			}
		}).collect::<Vec<String>>().join("\n\n"), &rpc_modules).as_bytes()).unwrap();
		println!("Types have been exported to {}", rpc_path.to_str().unwrap());
	}
}

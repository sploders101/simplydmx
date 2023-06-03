use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{LitStr, Type, Ident, ItemImpl};

struct InterfaceDesc {
	/// The name of the plugin
	name: String,
	/// List of events; (fn_name, event_name, payload_type)
	events: Vec<(Ident, LitStr, Type)>,
	/// The state object returned from the init function
	state_type: Option<Type>,
	impl_block: ItemImpl,
}

pub fn generate(attr: TokenStream, body: TokenStream) -> TokenStream {
	let mut input_iter = attr.into_iter();
	let name: LitStr = syn::parse2(TokenStream::from(
		input_iter.next().expect("Couldn't find plugin name"),
	))
	.expect("Invalid value for plugin name");
	let impl_block: ItemImpl = syn::parse2(body).expect("Attribute must be used on an impl block");

	let mut interface_desc = InterfaceDesc {
		name: name.value(),
		events: Vec::new(),
		state_type: None,
		impl_block,
	};

	while let Some(token) = input_iter.next() {
		match token.to_string().as_str() {
			"event" => {
				let event_name: Ident = syn::parse2(TokenStream::from(
					input_iter.next().expect("Couldn't find event name"),
				))
				.expect("Invalid event name");
				let event_name_str = event_name.to_string();
				let fn_name = Ident::new(&(String::from("emit_") + &event_name_str), event_name.span());
				let event_name = LitStr::new(&(name.value() + "." + &event_name_str), event_name.span());

				let event_type: Type = syn::parse2(TokenStream::from(
					input_iter.next().expect("Couldn't find event type"),
				))
				.expect("Invalid event type");

				interface_desc.events.push((fn_name, event_name, event_type));
			}
			other => panic!("Unrecognized descriptor '{}'.", other),
		}
	}

	return build_interface(interface_desc);
}



fn build_interface(interface: InterfaceDesc) -> TokenStream {
	let ident = interface.impl_block.self_ty;

	let state_extension = if let Some(state_type) = interface.state_type {
		quote! { state: #state_type, }
	} else {
		quote! { }
	};

	return quote! {
		struct #ident {
			plugin_interface: PluginInterface,
			#state_extension
		}
		impl #ident {
			pub fn init(interface: &PluginManager)
		}
	};
}

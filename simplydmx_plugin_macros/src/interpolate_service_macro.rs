use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
	parse::Parser, punctuated::Punctuated, Attribute, Expr, ExprLit, ExprParen, ExprTuple, FnArg,
	ImplItem, ImplItemMethod, ItemImpl, Lit, Pat, ReturnType, Token, Type,
};

use super::parsing_helpers::get_comma_delimited_strings;

static ARGERR: &str = "interpolate_service expects comma-separated list of description strings or (\"description\", \"type-id\") tuples.";

pub fn interpolate_service(attr: TokenStream, body: TokenStream) -> TokenStream {
	// Output aliases
	let arc = quote! {std::sync::Arc};

	// Gather standard documentation
	let mut docs = Punctuated::<Expr, Token![,]>::parse_terminated.parse(attr).expect("Improperly formatted outer macro usage.").into_iter();
	let service_id = docs.next().expect("ID not provided");
	let service_name = docs.next().expect("Name not provided");
	let service_description = docs.next().expect("Docs not provided");

	// Gather information from impl
	let impl_internals: ItemImpl = syn::parse(body)
	.expect("interpolate_service can only be used on an impl MyService statement");
	let name = impl_internals.self_ty;

	// Check for anything weird
	if impl_internals.unsafety.is_some() { panic!("Unsafety is not supported using interpolate_service"); }
	if impl_internals.generics.lt_token.is_some() || impl_internals.generics.where_clause.is_some() {
		panic!("Generics are not supported when using interpolate_service");
	}
	if impl_internals.trait_.is_some() { panic!("interpolate_service cannot be used on trait implementations"); }

	// Find inner type specified in another attribute on impl
	let mut inner_type: Option<Box<dyn ToTokens>> = None;
	let mut is_public = quote!{pub };
	for item in impl_internals.attrs {

		let is_inner = check_simple_attr(&item, "inner");
		let is_inner_raw = check_simple_attr(&item, "inner_raw");
		let is_private = check_simple_attr(&item, "private");

		if is_inner || is_inner_raw {
			if inner_type.is_some() { panic!("A service can have only one inner type") };
			let item_tokens: Pat = syn::parse(item.tokens.into()).expect("Invalid inner type");
			match item_tokens {
				Pat::Tuple(item_tokens) => {
					let inner_tokens = item_tokens.elems;
					match inner_tokens.len() {
						0 => {
							inner_type = Some(Box::new(quote! {()}));
						},
						1 => {
							if is_inner {
								inner_type = Some(Box::new(quote! {(#arc<#inner_tokens>)}));
							} else if is_inner_raw {
								inner_type = Some(Box::new(quote! {(#inner_tokens)}));
							}
						},
						_ => {
							if is_inner {
								inner_type = Some(Box::new(quote! {(#arc<(#inner_tokens)>)}));
							} else if is_inner_raw {
								inner_type = Some(Box::new(quote! {(#inner_tokens)}));
							}
						}
					}
				},
				_ => panic!("Unrecognized pattern on inner type attribute"),
			}
		} else if is_private {
			is_public = quote!{};
		}
	}

	// Expect required types
	let inner_type = inner_type.expect("Inner type not specified (ex: `#[inner(MyInnerDataType)]`)");

	// Go through impl items and look for the #[main] attribute
	let mut items: Vec<Box<dyn ToTokens>> = Vec::new();
	let mut service_implementation: Option<Box<dyn ToTokens>> = None;
	for item in impl_internals.items {
		match item {
			ImplItem::Method(item) => {
				let mut fn_added = false;
				for (i, attribute) in item.attrs.clone().into_iter().enumerate() {
					if check_simple_attr(&attribute, "service_main") {
						if let Some(_) = service_implementation {
							panic!("Cannot have two main functions in a service");
						} else {

							// Mark that we found the main function
							fn_added = true;

							// Remove marker attribute and add
							let mut cloned_item = item.clone();
							cloned_item.attrs.remove(i);
							items.push(Box::new(cloned_item.clone()));

							// Interpolate service calls from main
							service_implementation = Some(interpolate_service_main((*name).clone(), attribute.tokens.into(), cloned_item));

						}
					}
				}
				if !fn_added {
					items.push(Box::new(item));
				}
			},
			_ => {
				items.push(Box::new(item));
			}
		}
	}

	// Generate output
	let service_implementation = service_implementation.expect("Could not find entrypoint for service. Make sure to mark it with #[main(...)]");
	let gen = quote!{
		#[derive(Clone)]
		#is_public struct #name #inner_type;
		impl #name {
			#(#items)*
		}
		impl simplydmx_plugin_framework::Service for #name
		{
			fn get_id<'a>(&'a self) -> &'a str { #service_id }
			fn get_name<'a>(&'a self) -> &'a str { #service_name }
			fn get_description<'a>(&'a self) -> &'a str { #service_description }
			#service_implementation
		}
	};
	return gen.into();
}

fn check_simple_attr(attribute: &Attribute, expect: &str) -> bool {
	if attribute.path.leading_colon.is_some() { return false };
	if attribute.path.segments.len() != 1 { return false };
	return attribute.path.segments[0].ident.to_string() == expect;
}

/// Interpolates the inner main function of a service, creating functions related to documentation and
/// type casting
fn interpolate_service_main(outer_type: Type, attr: TokenStream, body: ImplItemMethod) -> Box<dyn ToTokens> {
	// Output aliases
	let pin = quote! {std::pin::Pin};
	let box_ = quote! {std::boxed::Box};
	let future = quote! {std::future::Future};
	let any = quote! {std::any::Any};
	let value = quote! {serde_json::Value};

	// Function internals
	let descriptions: ExprTuple = syn::parse(attr).expect(ARGERR);
	let descriptions = descriptions.elems.iter();
	// panic!("{}", attr);
	// let descriptions = Punctuated::<Expr, Token![,]>::parse_terminated.parse(attr).expect("Improperly formatted main macro usage.").into_iter();
	let internal_call = &body.sig.ident;
	let inject_await = if body.sig.asyncness.is_some() { quote! {.await} } else { quote! {} };

	let mut argument_names = Vec::<String>::new();

	// Vector of quote objects to use as arguments for the function this macro runs
	// on from the generic `call` implementation
	let mut internal_arguments = Vec::<Box<dyn ToTokens>>::new();
	let mut internal_arguments_json = Vec::<Box<dyn ToTokens>>::new();
	let mut internal_arguments_cbor = Vec::<Box<dyn ToTokens>>::new();
	let mut internal_argument_types = Vec::<Box<Type>>::new();

	// Number of arguments other than `self`
	let mut arg_count = 0;

	// Creates a list of argument conversions for use in a `quote!` macro comma-separated repeating statement,
	// along with some other metadata that will be helpful later.
	for arg in body.sig.inputs.iter() {
		match arg {

			// Make sure that any `self` argument is declared properly
			FnArg::Receiver(rec) => {
				if rec.reference.as_ref().is_some() {
					panic!("Service functions must take ownership of self. References are maintained through an Arc on self.0");
				}
			},

			// All non-`self` arguments...
			FnArg::Typed(arg) => {

				// Collect metadata about the current argument;
				let index = internal_arguments.len();
				let ty = *arg.ty.clone();
				internal_argument_types.push(Box::new(*arg.ty.clone()));

				// Log the name of the argument so we can recall it for documentation later
				if let Pat::Ident(ref pat) = *arg.pat {
					argument_names.push(pat.ident.to_string());
				} else {
					panic!("Unrecognized argument pattern");
				}

				// `call` implementation: Downcasts values to the correct type to call service-specific function
				internal_arguments.push(Box::new(quote! {
					match arguments[#index].downcast_ref::<#ty>() {
						Some(value) => #ty::clone(value),
						None => return Err(simplydmx_plugin_framework::CallServiceError::TypeValidationFailed),
					}
				}));
				internal_arguments_json.push(Box::new(quote! {
					match serde_json::from_value::<#ty>(#value::clone(&arguments[#index])) {
						Ok(arg) => arg,
						Err(_) => return Err(simplydmx_plugin_framework::CallServiceRPCError::DeserializationFailed),
					}
				}));
				internal_arguments_cbor.push(Box::new(quote! {
					match ciborium::de::from_reader::<'_, #ty, &[u8]>(&arguments[#index]) {
						Ok(arg) => arg,
						Err(_) => return Err(simplydmx_plugin_framework::CallServiceRPCError::DeserializationFailed),
					}
				}));
				arg_count += 1;
			},

		}
	}

	// Validate inputs
	let (expected_desc_count, has_return) = match body.sig.output.clone() {
		ReturnType::Default => (arg_count, false),
		ReturnType::Type(_, ret_type) => {
			let ret_type_str = ret_type.clone().into_token_stream().to_string();
			if ret_type_str == "()" {
				(arg_count, false)
			} else {
				// Append return value as an argument to allow normal parsing
				argument_names.push(String::from("return"));
				internal_argument_types.push(ret_type);
				(arg_count + 1, true)
			}
		},
	};
	if descriptions.len() != expected_desc_count {
		panic!("Expected {} description objects. Saw {} description(s).", expected_desc_count, descriptions.len());
	}

	let mut input_tokens = Vec::<Box<dyn ToTokens>>::new();

	// Iterate through the descriptions and build documentation about the function in code
	for (i, description) in descriptions.enumerate() {
		let desc_elements = match description {
			&Expr::Lit(ExprLit { lit: Lit::Str(ref str_lit), .. }) => vec![str_lit.clone()],
			&Expr::Paren(ExprParen { ref expr, .. }) => {
				if let &Expr::Lit(ExprLit { lit: Lit::Str(ref str_lit), .. }) = expr.as_ref() {
					vec![str_lit.clone()]
				} else {
					panic!("{}", ARGERR);
				}
			},
			&Expr::Tuple(ref description) => get_comma_delimited_strings(&description.elems, ARGERR),
			item => panic!("Unrecognized description item: {:?}", item),
		};
		let desc_length = desc_elements.len();
		if desc_length == 1 || desc_length == 2 {
			let id = argument_names.get(i).expect("Couldn't find argument name");
			// let desc_elements = ;
			let description = desc_elements.get(0).expect("Couldn't find the description");
			// let val_type = Type::clone(internal_argument_types.get(i).as_ref().expect("Couldn't find the internal argument type")).into_token_stream().to_string();
			let val_type = Type::clone(internal_argument_types.get(i).as_ref().expect("Couldn't find the internal argument type"));
			let val_type_str = val_type.to_token_stream().to_string();
			let type_id: Box<dyn ToTokens> = if let Some(type_id) = desc_elements.get(1) { Box::new(quote!{Some(#type_id)}) } else { Box::new(quote!{None}) };
			// let type_id: Box<dyn ToTokens> = Box::new(desc_elements.get(2).unwrap_or(quote!{None}));
			input_tokens.push(Box::new(quote! {
				simplydmx_plugin_framework::ServiceArgument {
					id: #id,
					description: #description,
					#[cfg(feature = "export-services")]
					val_type: {
						// Add the value of the alias here
						#[allow(nonstandard_style)]
						#[derive(tsify::Tsify)]
						#[serde(transparent)]
						struct FunctionArgument (#val_type);
						// This is not ideal. We want to use DECL[31..], but the compiler doesn't realize it's a static value
						// and throws an error. For some reason Box::leak doesn't work either.
						<FunctionArgument as tsify::Tsify>::DECL
					},
					#[cfg(not(feature = "export-services"))]
					val_type: #val_type_str,
					val_type_hint: #type_id,
				}
			}));
		} else {
			panic!("{}", ARGERR);
		}
	}

	let return_signature = if has_return {
		let arg_docs = input_tokens.pop();
		quote! { &Some(#arg_docs) }
	} else {
		quote! { &None }
	};

	return Box::new(quote! {
		fn get_signature<'a>(&'a self) -> (&'a [simplydmx_plugin_framework::ServiceArgument], &'a Option<simplydmx_plugin_framework::ServiceArgument>) {
			return (&[#(#input_tokens),*], #return_signature);
		}
		fn call<'a>(&'a self, arguments: Vec<#box_<dyn #any + Sync + Send>>) -> #pin<#box_<dyn #future<Output = Result<#box_<dyn #any + Sync + Send>, simplydmx_plugin_framework::CallServiceError>> + Send + 'a>> {
			async fn run(_self: #outer_type, arguments: Vec<#box_<dyn #any + Sync + Send>>) -> Result<#box_<dyn #any + Sync + Send>, simplydmx_plugin_framework::CallServiceError> {
				return Ok(#box_::new(#outer_type::#internal_call(_self, #(#internal_arguments),*)#inject_await));
			}

			return #box_::pin(run(#outer_type::clone(self), arguments));
		}
		fn call_json<'a>(&'a self, arguments: Vec<#value>) -> #pin<#box_<dyn #future<Output = Result<#value, simplydmx_plugin_framework::CallServiceRPCError>> + Send + 'a>> {
			async fn run(_self: #outer_type, arguments: Vec<serde_json::Value>) -> Result<serde_json::Value, simplydmx_plugin_framework::CallServiceRPCError> {
				let ret_val = serde_json::to_value(#outer_type::#internal_call(_self, #(#internal_arguments_json),*)#inject_await);
				return match ret_val {
					Ok(ret_val) => Ok(ret_val),
					Err(_) => Err(simplydmx_plugin_framework::CallServiceRPCError::SerializationFailed),
				};
			}

			return #box_::pin(run(#outer_type::clone(&self), arguments));
		}

		fn call_cbor<'a>(&'a self, arguments: Vec<Vec<u8>>) -> #pin<#box_<dyn #future<Output = Result<Vec<u8>, simplydmx_plugin_framework::CallServiceRPCError>> + Send + 'a>> {
			async fn run(_self: #outer_type, arguments: Vec<Vec<u8>>) -> Result<Vec<u8>, simplydmx_plugin_framework::CallServiceRPCError> {
				let mut ret_val = Vec::<u8>::new();
				let ret_val_result = ciborium::ser::into_writer(&#outer_type::#internal_call(_self, #(#internal_arguments_cbor),*)#inject_await, &mut ret_val);
				return match ret_val_result {
					Ok(_) => Ok(ret_val),
					Err(_) => Err(simplydmx_plugin_framework::CallServiceRPCError::SerializationFailed),
				}
			}

			return #box_::pin(run(#outer_type::clone(&self), arguments));
		}
	});
}

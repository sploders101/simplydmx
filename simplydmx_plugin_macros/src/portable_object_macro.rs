use proc_macro::TokenStream;
use quote::{
	ToTokens, quote,
};
use std::collections::HashSet;
use proc_macro2::{Delimiter, TokenTree, Ident, Span};
use syn::{
	punctuated::Punctuated,
	parse::Parser,
	Attribute,
	Meta,
	Lit,
};


fn get_ts_docs(attributes: &Vec<Attribute>) -> proc_macro2::TokenStream {
	let doc_lines: Vec<String> = attributes.iter().filter_map(|attr| {
		if attr.path.segments[0].ident == "doc" {
			let meta = attr.parse_meta();
			return match meta {
				Ok(meta) => match meta {
					Meta::NameValue(meta) => match meta.lit {
						Lit::Str(meta) => Some(String::from(" * ") + meta.value().trim()),
						_ => Some(String::from(" *")),
					},
					_ => Some(String::from(" *")),
				},
				Err(_) => Some(String::from(" *")),
			};
		} else {
			return None;
		}
	}).collect();

	if doc_lines.len() == 0 {
		return quote!{&None};
	} else {
		let formatted_body = String::from("/**\n") + &doc_lines.join("\n") + "\n */";
		return quote! {&Some(#formatted_body)};
	}
}


pub fn portable_object(_attr: TokenStream, body: TokenStream) -> TokenStream {
	let (modified_body, ident, docs): (proc_macro2::TokenStream, Ident, proc_macro2::TokenStream) = if let Ok(mut input) = syn::parse::<syn::ItemStruct>(body.clone()) {
		if let Some(error) = edit_attributes(&mut input.attrs) { return error; }
		let ident = input.ident.clone();
		let docs = get_ts_docs(&input.attrs);
		(input.into_token_stream().into(), ident, docs)
	} else if let Ok(mut input) = syn::parse::<syn::ItemEnum>(body.clone()) {
		if let Some(error) = edit_attributes(&mut input.attrs) { return error; }
		let ident = input.ident.clone();
		let docs = get_ts_docs(&input.attrs);
		(input.into_token_stream().into(), ident, docs)
	} else if let Ok(input) = syn::parse::<syn::ItemType>(body.clone()) {
		// These items cannot implement traits since they are just aliases, so create a transparent struct to implement on
		let ident = input.ident.clone();
		let ident_str = ident.to_string();
		let generics = input.generics.clone();
		let alias_value = input.ty.clone();
		let docs = get_ts_docs(&input.attrs);
		let body = input.into_token_stream();
		let portable_ident = Ident::new(&format!("PORTABLETYPE__{}", &ident_str), Span::call_site());
		let portable_ident_hidden = Ident::new(&format!("PORTABLETYPE__WRAPPER__{}", &ident_str), Span::call_site());

		return quote! {
			#body

			// Add the value of the alias here
			#[cfg(feature = "export-services")]
			#[allow(nonstandard_style)]
			#[derive(tsify::Tsify)]
			#[serde(transparent)]
			#[serde(rename = #ident_str)]
			struct #portable_ident_hidden #generics (#alias_value);

			#[cfg(feature = "export-services")]
			#[allow(nonstandard_style)]
			#[linkme::distributed_slice(crate::init::exporter::PORTABLETYPE)]
			static #portable_ident: (&'static str, &'static str, &'static Option<&'static str>) = (#ident_str, <#portable_ident_hidden as tsify::Tsify>::DECL, #docs);
		}.into();
	} else {
		panic!("Data type not recognized");
	};

	let ident_str = ident.to_string();
	let portable_ident = Ident::new(&format!("PORTABLETYPE__{}", &ident_str), Span::call_site());

	return quote! {
		#modified_body

		#[cfg(feature = "export-services")]
		#[allow(nonstandard_style)]
		#[linkme::distributed_slice(crate::init::exporter::PORTABLETYPE)]
		static #portable_ident: (&'static str, &'static str, &'static Option<&'static str>) = (#ident_str, <#ident as tsify::Tsify>::DECL, #docs);
	}.into();
}

fn edit_attributes(attrs: &mut Vec<syn::Attribute>) -> Option<TokenStream> {
	let mut all_derived_traits = HashSet::<syn::Path>::new();
	let mut derive_attributes = Vec::<syn::Attribute>::new();
	attrs.retain(|attr| {
		if !attr.path.is_ident("derive") {
			return true;
		}
		derive_attributes.push(attr.clone());
		return false;
	});
	for derive in derive_attributes {
		let mut tokens = derive.tokens.clone().into_iter();
		match [tokens.next(), tokens.next()] {
			[Some(TokenTree::Group(group)), None]
				if group.delimiter() == Delimiter::Parenthesis =>
			{
				match Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
					.parse2(group.stream())
				{
					Ok(derived_traits) => all_derived_traits.extend(derived_traits),
					Err(e) => return Some(e.into_compile_error().into()),
				}
			}
			_ => {
				return Some(syn::Error::new_spanned(derive, "malformed derive")
					.into_compile_error()
					.into())
			}
		}
	}

	all_derived_traits.extend([
		#[cfg(feature = "tsify")]
		syn::parse_quote!(tsify::Tsify),
		syn::parse_quote!(Debug),
		syn::parse_quote!(Clone),
		syn::parse_quote!(serde::Serialize),
		syn::parse_quote!(serde::Deserialize),
	]);

	let all_derived_traits = all_derived_traits.into_iter();
	attrs.insert(0, syn::parse_quote! {
		#[derive( #(#all_derived_traits),* )]
	});

	#[cfg(feature = "tsify-wasm-abi")]
	attrs.insert(1, syn::parse_quote!{#[tsify(into_wasm_abi, from_wasm_abi)]});

	return None;
}

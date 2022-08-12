use proc_macro::TokenStream;
use quote::{
    ToTokens,
};
use std::collections::HashSet;
use proc_macro2::{Delimiter, TokenTree};
use syn::{
    punctuated::Punctuated,
    parse::Parser,
};

pub fn portable_object(_attr: TokenStream, body: TokenStream) -> TokenStream {
	if let Ok(mut input) = syn::parse::<syn::ItemStruct>(body.clone()) {
		if let Some(error) = edit_attributes(&mut input.attrs) { return error; }
    	return input.into_token_stream().into();
	} else if let Ok(mut input) = syn::parse::<syn::ItemEnum>(body.clone()) {
		if let Some(error) = edit_attributes(&mut input.attrs) { return error; }
		return input.into_token_stream().into();
	} else {
		panic!("Data type not recognized");
	}
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
        syn::parse_quote!(Debug),
        syn::parse_quote!(Clone),
        syn::parse_quote!(serde::Serialize),
        syn::parse_quote!(serde::Deserialize),
    ]);

    let all_derived_traits = all_derived_traits.into_iter();
    attrs.insert(0, syn::parse_quote! {
        #[derive( #(#all_derived_traits),* )]
    });

	return None;
}

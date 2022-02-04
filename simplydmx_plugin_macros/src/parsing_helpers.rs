use quote::{
	ToTokens,
	quote,
};
use syn::{
    punctuated::Punctuated,
    Expr,
    Token,
    Lit,
    LitStr,
	Type,
};

pub fn get_comma_delimited_strings(input: &Punctuated<Expr, Token![,]>, error_msg: &str) -> Vec<LitStr> {
    let mut strings = Vec::new();

    for string in input.iter() {
		if let Expr::Lit(string) = string {
			if let Lit::Str(ref string) = string.lit {
				strings.push(LitStr::clone(string));
			} else {
				panic!("{}", error_msg);
			}
		} else {
			panic!("{}", error_msg);
		}
	}

    return strings;
}

pub fn get_typedoc(in_type: Type) -> Box<dyn ToTokens> {

	// Alias setup
	let internals = quote! {simplydmx_plugin_framework::services::internals};
	let modifiers = quote! {#internals::ServiceArgumentModifiers};
	let required = quote! {#modifiers::Required};
	let optional = quote! {#modifiers::Optional};
	let vector = quote! {#modifiers::Vector};
	let types = quote! {#internals::ServiceDataTypes};

	// Type documentation
	let in_type = in_type.into_token_stream().to_string();
	return Box::new(match &in_type as &str {

		// Required
		"u8" => quote! {#required(#types::U8)},
		"u16" => quote! {#required(#types::U16)},
		"u32" => quote! {#required(#types::U32)},
		"i8" => quote! {#required(#types::I8)},
		"i16" => quote! {#required(#types::I16)},
		"i32" => quote! {#required(#types::I32)},
		"String" => quote! {#required(#types::String)},

		// Optional
		"Option<u8>" => quote! {#optional(#types::U8)},
		"Option<u16>" => quote! {#optional(#types::U16)},
		"Option<u32>" => quote! {#optional(#types::U32)},
		"Option<i8>" => quote! {#optional(#types::I8)},
		"Option<i16>" => quote! {#optional(#types::I16)},
		"Option<i32>" => quote! {#optional(#types::I32)},
		"Option<String>" => quote! {#optional(#types::String)},

		// List
		"Vec<u8>" => quote! {#vector(#types::U8)},
		"Vec<u16>" => quote! {#vector(#types::U16)},
		"Vec<u32>" => quote! {#vector(#types::U32)},
		"Vec<i8>" => quote! {#vector(#types::I8)},
		"Vec<i16>" => quote! {#vector(#types::I16)},
		"Vec<i32>" => quote! {#vector(#types::I32)},
		"Vec<String>" => quote! {#vector(#types::String)},

		// Unknown
		type_string => panic!("Type {} is not recognized as a valid service input.", type_string),
	});
}

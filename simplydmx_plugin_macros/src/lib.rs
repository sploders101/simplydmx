extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{
    quote,
    ToTokens,
};
use syn::{
    ItemFn,
    FnArg,
    punctuated::Punctuated,
    parse::Parser,
    Expr,
    Token,
    Pat,
    Type,
    ReturnType,
};

mod parsing_helpers;
use parsing_helpers::{
    get_comma_delimited_strings,
    get_typedoc,
};

static ARGERR: &str = "interpolate_service expects comma-separated list of description strings or (\"name\", \"description\", \"type-id\") tuples.";

#[proc_macro_derive(Service)]
pub fn service_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let internals = quote! {simplydmx_plugin_framework::services::internals};

    let gen = quote! {
        impl Service for #name {
            fn get_id<'a>(&'a self) -> &'a str { self.id }
            fn get_name<'a>(&'a self) -> &'a str { self.name }
            fn get_description<'a>(&'a self) -> &'a str { self.description }
            fn get_signature<'a>(&'a self) -> (&'a [#internals::ServiceArgument], Option<&'a #internals::ServiceArgument>) { #name::get_signature_internal(self) }
            fn call(&self, arguments: Vec<Box<dyn std::any::Any>>) -> Result<Box<dyn std::any::Any>, #internals::CallServiceError> { #name::call_native_internal(self, arguments) }
            fn call_json(&self, arguments: Vec<serde_json::Value>) -> Result<serde_json::Value, #internals::CallServiceError> { #name::call_json_internal(self, arguments) }
        }
    };
    return gen.into();
}

#[proc_macro_attribute]
pub fn interpolate_service(attr: TokenStream, body: TokenStream) -> TokenStream {

    // Aliases
    let internals = quote! {simplydmx_plugin_framework::services::internals};

    // Function internals
    let descriptions = Punctuated::<Expr, Token![,]>::parse_terminated.parse(attr)
        .expect(ARGERR);
    let fn_internals: ItemFn = syn::parse(body)
        .expect("interpolate_service can only be used on a function in an impl MyService statement");
    let internal_call = &fn_internals.sig.ident;

    let mut argument_names = Vec::<String>::new();

    // Vector of quote objects to use as arguments for the function this macro runs
    // on from the generic `call` implementation
    let mut internal_arguments = Vec::<Box<dyn ToTokens>>::new();
    let mut internal_argument_types = Vec::<Box<Type>>::new();

    // Number of arguments other than `self`
    let mut arg_count = 0;

    // Creates a list of argument conversions for use in a `quote!` macro comma-separated repeating statement,
    // along with some other metadata that will be helpful later.
    for arg in fn_internals.sig.inputs.iter() {
        match arg {

            // Make sure that any `self` argument is declared properly
            FnArg::Receiver(rec) => {
                rec.reference.as_ref().expect("Service functions cannot take ownership of self");
                if rec.mutability.is_some() {
                    panic!("Service functions cannot take a mutable reference. If mutability is required, use a lock.");
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
                        Some(value) => {
                            #ty::clone(value)
                        },
                        None => {
                            return Err(#internals::CallServiceError::TypeValidationFailed);
                        }
                    }
                }));
                arg_count += 1;
            },

        }
    }

    // Validate inputs
    let (expected_desc_count, has_return) = match fn_internals.sig.output.clone() {
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
        panic!("There should be the same number of interpolate_service description arugments as function arguments");
    }

    let mut input_tokens = Vec::<Box<dyn ToTokens>>::new();

    // Iterate through the descriptions and build documentation about the function in code
    for (i, description) in descriptions.iter().enumerate() {
        if let Expr::Tuple(description) = description {
            let desc_length = description.elems.len();
            if desc_length == 2 || desc_length == 3 {
                let id = argument_names.get(i).expect("Couldn't find argument name");
                let desc_elements = get_comma_delimited_strings(&description.elems, ARGERR);
                let name = desc_elements.get(0).expect("Couldn't find human-readable name");
                let description = desc_elements.get(1).expect("Couldn't find the description");
                let val_type = get_typedoc(Type::clone(internal_argument_types.get(i).as_ref().expect("Couldn't find the internal argument type")));
                let type_id: Box<dyn ToTokens> = if let Some(stuff) = desc_elements.get(2) { Box::new(quote!{Some(String::from(#stuff))}) } else { Box::new(quote!{None}) };
                // let type_id: Box<dyn ToTokens> = Box::new(desc_elements.get(2).unwrap_or(quote!{None}));
                input_tokens.push(Box::new(quote! {
                    #internals::ServiceArgument {
                        id: String::from(#id),
                        name: String::from(#name),
                        description: String::from(#description),
                        val_type: #val_type,
                        val_type_id: #type_id,
                    }
                }));
            } else {
                panic!("{}", ARGERR);
            }
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

    let gen = quote! {
        #fn_internals

        pub fn get_signature_internal(&self) -> (&'static [#internals::ServiceArgument], &'static Option<#internals::ServiceArgument>) {
            return (&[#(#input_tokens),*], #return_signature);
        }

        pub fn call_native_internal(&self, arguments: Vec<Box<dyn std::any::Any>>) -> Result<Box<dyn std::any::Any>, #internals::CallServiceError> {
            return Ok(Box::new(self.#internal_call(#(#internal_arguments),*)));
        }

        pub fn call_json_internal(&self, arguments: Vec<serde_json::Value>) -> Result<serde_json::Value, #internals::CallServiceError> {
            return Ok(serde_json::Value::Null);
        }
    };
    return gen.into();
}

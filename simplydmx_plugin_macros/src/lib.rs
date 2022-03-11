extern crate proc_macro;

use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::{
    quote,
    ToTokens,
};
use syn::{
    FnArg,
    punctuated::Punctuated,
    parse::Parser,
    Expr,
    Token,
    Pat,
    Type,
    ReturnType,
    ItemImpl,
    ImplItem,
    Attribute,
    ExprTuple,
    ImplItemMethod,
};

mod parsing_helpers;
use parsing_helpers::{
    get_comma_delimited_strings,
    get_typedoc,
};

static ARGERR: &str = "interpolate_service expects comma-separated list of description strings or (\"name\", \"description\", \"type-id\") tuples.";

lazy_static! {
    static ref internals: quote::__private::TokenStream = quote! {simplydmx_plugin_framework::services::internals};
    static ref pin: quote::__private::TokenStream = quote! {std::pin::Pin};
    static ref box_: quote::__private::TokenStream = quote! {std::boxed::Box};
    static ref future: quote::__private::TokenStream = quote! {std::future::Future};
    static ref any: quote::__private::TokenStream = quote! {std::any::Any};
    static ref value: quote::__private::TokenStream = quote! {serde_json::Value};
    static ref arc: quote::__private::TokenStream = quote! {std::sync::Arc};
}

#[proc_macro_derive(Service)]
pub fn service_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let gen = quote! {
        impl Service for #name {
            fn get_id<'a>(&'a self) -> &'a str { #name::get_service_id_internal() }
            fn get_name<'a>(&'a self) -> &'a str { #name::get_service_name_internal() }
            fn get_description<'a>(&'a self) -> &'a str { #name::get_service_description_internal() }
            fn get_signature<'a>(&'a self) -> (&'a [#internals::ServiceArgument], &'a Option<#internals::ServiceArgument>) { #name::get_service_signature_internal(self) }
            fn call<'a>(&'a self, arguments: Vec<#box_<dyn #any>>) -> #pin<#box_<dyn #future<Output = Result<#box_<dyn #any>, #internals::CallServiceError>> + Send + 'a>> { #name::call_service_native_internal(self, arguments) }
            fn call_json<'a>(&'a self, arguments: Vec<Value>) -> #pin<#box_<dyn #future<Output = Result<Value, #internals::CallServiceJSONError>> + Send + 'a>> { #name::call_service_json_internal(self, arguments) }
        }
    };
    return gen.into();
}

#[proc_macro_attribute]
pub fn interpolate_service(attr: TokenStream, body: TokenStream) -> TokenStream {

    // Gather standard documentation
    let mut docs = Punctuated::<Expr, Token![,]>::parse_terminated.parse(attr).expect("Improperly formatted outer macro usage.").into_iter();
    let inner_type = docs.next().expect("Inner type not provided");
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

    // Go through impl items and look for the #[main] attribute
    let mut found_main = false;
    let mut items: Vec<Box<dyn ToTokens>> = Vec::new();
    for item in impl_internals.items {
        match item {
            ImplItem::Method(item) => {
                for (i, attribute) in item.attrs.clone().into_iter().enumerate() {
                    if check_is_main(&attribute) {
                        if found_main {
                            panic!("Cannot have two main functions in a service");
                        } else {
                            found_main = true;

                            // Interpolate service calls from main
                            let mut cloned_item = item.clone();
                            cloned_item.attrs.remove(i);
                            items.push(interpolate_service_main((*name).clone(), inner_type.clone(), attribute.tokens.into(), cloned_item));
                        }
                    }
                }
            },
            _ => {
                items.push(Box::new(item));
            }
        }
    }

    // Generate output
    let gen = quote!{
        struct #name (#arc<#inner_type>);
        impl #name {
            #(#items)*
        }
        impl Clone for #name {
            fn clone(&self) -> Self {
                return #name (#arc::clone(&self.0));
            }
        }
        impl #internals::Service for #name {
            fn get_id<'a>(&'a self) -> &'a str { #service_id }
            fn get_name<'a>(&'a self) -> &'a str { #service_name }
            fn get_description<'a>(&'a self) -> &'a str { #service_description }
            fn get_signature<'a>(&'a self) -> (&'a [#internals::ServiceArgument], &'a Option<#internals::ServiceArgument>) { #name::get_service_signature_internal(self) }
            fn call<'a>(&'a self, arguments: Vec<#box_<dyn #any + Send>>) -> #pin<#box_<dyn #future<Output = Result<#box_<dyn #any + Send>, #internals::CallServiceError>> + Send + 'a>> { #name::call_service_native_internal(self, arguments) }
            fn call_json<'a>(&'a self, arguments: Vec<#value>) -> #pin<#box_<dyn #future<Output = Result<#value, #internals::CallServiceJSONError>> + Send + 'a>> { #name::call_service_json_internal(self, arguments) }
        }
    };
    return gen.into();
}

fn check_is_main(attribute: &Attribute) -> bool {
    if attribute.path.leading_colon.is_some() { return false };
    if attribute.path.segments.len() != 1 { return false };
    return attribute.path.segments[0].ident.to_string() == "main";
}

/// Interpolates the inner main function of a service, creating functions related to documentation and
/// type casting
fn interpolate_service_main(outer_type: Type, _inner_type: Expr, attr: TokenStream, body: ImplItemMethod) -> Box<dyn ToTokens> {

    // Function internals
    let descriptions: ExprTuple = syn::parse(attr).expect(ARGERR);
    let descriptions = descriptions.elems.iter();
    // panic!("{}", attr);
    // let descriptions = Punctuated::<Expr, Token![,]>::parse_terminated.parse(attr).expect("Improperly formatted main macro usage.").into_iter();
    let internal_call = &body.sig.ident;

    let mut argument_names = Vec::<String>::new();

    // Vector of quote objects to use as arguments for the function this macro runs
    // on from the generic `call` implementation
    let mut internal_arguments = Vec::<Box<dyn ToTokens>>::new();
    let mut internal_arguments_json = Vec::<Box<dyn ToTokens>>::new();
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
                        None => return Err(#internals::CallServiceError::TypeValidationFailed),
                    }
                }));
                internal_arguments_json.push(Box::new(quote! {
                    match serde_json::from_value::<#ty>(#value::clone(&arguments[#index])) {
                        Ok(arg) => arg,
                        Err(_) => return Err(#internals::CallServiceJSONError::DeserializationFailed),
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
        if let Expr::Tuple(description) = description {
            let desc_length = description.elems.len();
            if desc_length == 2 || desc_length == 3 {
                let id = argument_names.get(i).expect("Couldn't find argument name");
                let desc_elements = get_comma_delimited_strings(&description.elems, ARGERR);
                let name = desc_elements.get(0).expect("Couldn't find human-readable name");
                let description = desc_elements.get(1).expect("Couldn't find the description");
                let val_type = get_typedoc(Type::clone(internal_argument_types.get(i).as_ref().expect("Couldn't find the internal argument type")));
                let type_id: Box<dyn ToTokens> = if let Some(type_id) = desc_elements.get(2) { Box::new(quote!{Some(#type_id)}) } else { Box::new(quote!{None}) };
                // let type_id: Box<dyn ToTokens> = Box::new(desc_elements.get(2).unwrap_or(quote!{None}));
                input_tokens.push(Box::new(quote! {
                    #internals::ServiceArgument {
                        id: #id,
                        name: #name,
                        description: #description,
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

    return Box::new(quote! {
        #body

        pub fn get_service_signature_internal(&self) -> (&'static [#internals::ServiceArgument<'static>], &'static Option<#internals::ServiceArgument<'static>>) {
            return (&[#(#input_tokens),*], #return_signature);
        }

        pub fn call_service_native_internal<'a>(&self, arguments: Vec<#box_<dyn #any + Send>>) -> #pin<#box_<dyn #future<Output = Result<#box_<dyn #any + Send>, #internals::CallServiceError>> + Send + 'a>>
        where
            Self: Sync + 'a
        {
            async fn run(_self: #outer_type, arguments: Vec<#box_<dyn #any + Send>>) -> Result<#box_<dyn #any + Send>, #internals::CallServiceError> {
                return Ok(#box_::new(#outer_type::#internal_call(_self, #(#internal_arguments),*)));
            }

            return #box_::pin(run(#outer_type::clone(&self), arguments));
        }

        pub fn call_service_json_internal<'a>(&self, arguments: Vec<serde_json::Value>) -> #pin<#box_<dyn #future<Output = Result<serde_json::Value, #internals::CallServiceJSONError>> + Send + 'a>>
        where
            Self: Sync + 'a
        {
            async fn run(_self: #outer_type, arguments: Vec<serde_json::Value>) -> Result<serde_json::Value, #internals::CallServiceJSONError> {
                let ret_val = serde_json::to_value(#outer_type::#internal_call(_self, #(#internal_arguments_json),*));
                return match ret_val {
                    Ok(ret_val) => Ok(ret_val),
                    Err(_) => Err(#internals::CallServiceJSONError::SerializationFailed),
                };
            }

            return #box_::pin(run(#outer_type::clone(&self), arguments));
        }
    });
}

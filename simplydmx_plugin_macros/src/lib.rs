extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    ItemFn,
    FnArg,
};

#[proc_macro_derive(Service)]
pub fn service_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let gen = quote! {
        impl Service for #name {
            fn get_id<'a>(&'a self) -> &'a str { self.id }
            fn get_name<'a>(&'a self) -> &'a str { self.name }
            fn get_description<'a>(&'a self) -> &'a str { self.description }
            fn get_signature<'a>(&'a self) -> (&[simplydmx_plugin_framework::services::internals::ServiceArgument], Option<simplydmx_plugin_framework::services::internals::ServiceArgument>) { #name::get_signature_internal(self) }
            fn call(&self, arguments: Vec<Box<dyn std::any::Any>>) -> Result<Box<dyn std::any::Any>, simplydmx_plugin_framework::services::internals::CallServiceError> { #name::call_native_internal(self, arguments) }
            fn call_json(&self, arguments: Vec<serde_json::Value>) -> Result<serde_json::Value, simplydmx_plugin_framework::services::internals::CallServiceError> { #name::call_json_internal(self, arguments) }
        }
    };
    return gen.into();
}

#[proc_macro_attribute]
pub fn interpolate_service(_: TokenStream, body: TokenStream) -> TokenStream {
    let internals: ItemFn = syn::parse(body).unwrap();

    let internal_call = &internals.sig.ident;

    // Vector of quote objects to use as arguments for the function this macro runs
    // on from the generic `call` implementation
    let mut internal_arguments = Vec::<Box<dyn ToTokens>>::new();

    // Iterate over input arguments
    for arg in internals.sig.inputs.iter() {
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
                let ty = arg.ty.clone();

                // `call` implementation: Downcasts values to the correct type to call service-specific function
                internal_arguments.push(Box::new(quote! {
                    match arguments[#index].downcast_ref::<#ty>() {
                        Some(value) => {
                            #ty::clone(value)
                        },
                        None => {
                            return Err(simplydmx_plugin_framework::services::internals::CallServiceError::TypeValidationFailed);
                        }
                    }
                }));
            },

        }
    }

    let gen = quote! {
        #internals

        pub fn get_signature_types() -> (&[simplydmx_plugin_framework::services::internals::ServiceArgument], Option<simplydmx_plugin_framework::services::internals::ServiceArgument>) {
            return ([], None);
        }

        pub fn call_native_internal(&self, arguments: Vec<Box<dyn std::any::Any>>) -> Result<Box<dyn std::any::Any>, simplydmx_plugin_framework::services::internals::CallServiceError> {
            return Ok(Box::new(self.#internal_call(#(#internal_arguments),*)));
        }

        pub fn call_json_internal(&self, arguments: Vec<serde_json::Value>) -> Result<serde_json::Value, simplydmx_plugin_framework::services::internals::CallServiceError> {
            return Ok(serde_json::Value::Null);
        }
    };
    return gen.into();
}

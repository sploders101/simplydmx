extern crate proc_macro;
use proc_macro::TokenStream;

mod interpolate_service_macro;
mod parsing_helpers;
mod portable_object_macro;


#[proc_macro_attribute]
pub fn interpolate_service(attr: TokenStream, body: TokenStream) -> TokenStream {
    return interpolate_service_macro::interpolate_service(attr, body);
}

#[proc_macro_attribute]
pub fn portable(attr: TokenStream, body: TokenStream) -> TokenStream {
    return portable_object_macro::portable_object(attr, body);
}

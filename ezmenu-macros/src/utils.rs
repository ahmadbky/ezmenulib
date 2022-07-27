use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::quote;
use syn::{parse::Parse, Attribute};

// The library name might change
pub fn get_lib_root() -> TokenStream {
    quote!(::ezmenulib)
}

pub fn get_attr<'a>(attrs: &'a [Attribute], ident: &str) -> Option<&'a Attribute> {
    attrs
        .iter()
        .find(|attr| attr.path.segments.iter().any(|seg| seg.ident == ident))
}

pub fn get_attr_with_args<A: Parse>(attrs: &[Attribute], ident: &str) -> Option<A> {
    get_attr(attrs, ident).map(|attr| {
        attr.parse_args()
            .unwrap_or_else(|e| abort!(e.span(), "invalid attribute: {}", e))
    })
}

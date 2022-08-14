use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn build_menu(_input: DeriveInput) -> TokenStream {
    quote!()
}

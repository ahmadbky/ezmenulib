use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

extern crate proc_macro as pm;

mod format;
mod prompted;
mod utils;

use self::prompted::build_prompted;

#[proc_macro_attribute]
#[doc(hidden)]
pub fn __debug_attr(_attr: pm::TokenStream, item: pm::TokenStream) -> pm::TokenStream {
    println!("{item:#?}");
    item
}

#[proc_macro_derive(__Debug_Derv)]
#[doc(hidden)]
pub fn __debug_derive(item: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    for attr in &input.attrs {
        println!("{:#?}", attr.parse_meta())
    }
    pm::TokenStream::new()
}

#[proc_macro_error]
#[proc_macro_derive(Prompted, attributes(prompt))]
pub fn derive_prompted(item: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    build_prompted(input).into()
}

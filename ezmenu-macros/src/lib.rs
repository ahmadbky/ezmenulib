//! Ezmenu attribute and derive macros definition.

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unreachable_pub,
    unused_lifetimes,
    future_incompatible
)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod pretend;
mod bound;
mod format;
mod generics;
mod kw;
mod menu;
mod prompted;
mod utils;

use self::{bound::{build_bound, BoundArgs}, menu::build_menu, prompted::build_prompted};

#[proc_macro_attribute]
#[doc(hidden)]
pub fn __debug_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr={attr:#?}");
    println!("item={item:#?}");
    item
}

#[proc_macro_derive(__Debug_Derv)]
#[doc(hidden)]
pub fn __debug_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    for attr in &input.attrs {
        println!("{:#?}", attr.parse_meta())
    }
    TokenStream::new()
}

/// Prompted macro
#[proc_macro_error]
#[proc_macro_derive(Prompted, attributes(prompt))]
pub fn derive_prompted(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    build_prompted(input).into()
}

/// Menu macro
#[proc_macro_error]
#[proc_macro_derive(Menu, attributes(menu))]
pub fn derive_menu(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    build_menu(input).into()
}

/// Bound attribute macro
#[proc_macro_error]
#[proc_macro_attribute]
pub fn bound(attr: TokenStream, item: TokenStream) -> TokenStream {
    let tui = parse_macro_input!(attr as BoundArgs);
    let input = parse_macro_input!(item as syn::ItemFn);
    build_bound(tui, input).into()
}

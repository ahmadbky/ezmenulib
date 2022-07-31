use proc_macro2::TokenStream;
use proc_macro_error::{abort_call_site, proc_macro_error};
use syn::{
    parse_macro_input, punctuated::Punctuated, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    Fields, Ident, Token, Variant,
};

extern crate proc_macro as pm;

mod select;
mod utils;
mod format;

use self::select::build_select;

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
#[proc_macro_derive(Select, attributes(select))]
pub fn derive_selectable(item: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    build_select(input).into()
}

#[proc_macro_error]
#[proc_macro_derive(Menu, attributes(menu))]
pub fn derive_menu(item: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    match input.data {
        Data::Struct(DataStruct { fields, .. }) => {
            build_menu_struct(input.ident, input.attrs, fields)
        }
        Data::Enum(DataEnum { variants, .. }) => {
            build_menu_enum(input.ident, input.attrs, variants)
        }
        _ => abort_call_site!("derive(Menu) only supports `enum` and non-unit `struct` types."),
    }
    .into()
}

fn build_menu_struct(name: Ident, attrs: Vec<Attribute>, fields: Fields) -> TokenStream {
    TokenStream::new()
}

fn build_menu_enum(
    name: Ident,
    attrs: Vec<Attribute>,
    variants: Punctuated<Variant, Token![,]>,
) -> TokenStream {
    TokenStream::new()
}

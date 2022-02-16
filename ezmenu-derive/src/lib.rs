//! This crate is a derive procedural macro for `EZMenu` crate.
//! It should not be used directly. You must use the [`ezmenu`](https://docs.rs/ezmenu) crate.

mod struct_field;
mod struct_impl;

mod utils;

extern crate proc_macro as pm;

use crate::struct_field::{FieldFormatting, FieldMenuInit};
use crate::struct_impl::MenuInit;
pub(crate) use utils::*;

use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed,
    Ident,
};

// TODO: parser attribute
#[proc_macro_attribute]
#[proc_macro_error]
pub fn parser(_attr: pm::TokenStream, _ts: pm::TokenStream) -> pm::TokenStream {
    pm::TokenStream::new()
}

#[cfg(feature = "parsed_attr")]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn parsed(_attr: pm::TokenStream, ts: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(ts as DeriveInput);
    if let Data::Enum(e) = &input.data {
        build_parsed_enum(&input, e)
    } else {
        abort!(
            input,
            "ezmenu::parsed macro attribute only works on unit-like enums."
        )
    }
    .into()
}

fn build_parsed_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let ident = &input.ident;

    let inputs = data
        .variants
        .iter()
        .map(|var| var.ident.to_string().to_lowercase());
    let outputs = data.variants.iter().map(|var| &var.ident);

    quote! {
        #input
        impl ::std::str::FromStr for #ident {
            type Err = ::ezmenu::MenuError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    #(#inputs => Ok(Self::#outputs),)*
                    _ => Err(::ezmenu::MenuError::Other(
                        // necessary to provide error because default value can be provided
                        Box::new(format!("unrecognized input for `{}`", s))))
                }
            }
        }
    }
}

#[cfg(feature = "derive")]
#[proc_macro_derive(Menu, attributes(menu))]
#[proc_macro_error]
pub fn build_menu(ts: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(ts as DeriveInput);
    match input.data {
        Data::Enum(_e) => todo!("derive on enum soon"),
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => build_struct(input.ident, input.attrs, fields),
        _ => abort_call_site!("Menu derive supports only non-tuple structs and unit-like enums."),
    }
    .into()
}

fn def_init(menu_desc: MenuInit) -> TokenStream {
    let fields = menu_desc.fields.iter().map(|field| &field.kind);
    quote! {
        pub fn from_menu() -> ::ezmenu::MenuResult<Self> {
            let mut menu = ::ezmenu::StructMenu::default()
                #menu_desc;
            Ok(Self {#(
                #fields
            )*})
        }

        pub fn from_menu_unwrap() -> Self {
            Self::from_menu()
                .expect("An error occurred while processing menu")
        }
    }
}

fn build_struct(name: Ident, attrs: Vec<Attribute>, fields: FieldsNamed) -> TokenStream {
    // optional menu attr of the struct
    let struct_attr = get_meta_attr(attrs);
    // fields of the struct mapped to menu fields description
    let fields = fields.named.into_iter().map(FieldMenuInit::from).collect();

    let init = def_init(MenuInit::new(struct_attr, fields));

    quote! {
        impl #name {
            #init
        }
    }
}

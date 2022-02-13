//! This crate is a derive procedural macro for `EZMenu` crate.
//! It should not be used directly. You must use the [`ezmenu`](https://docs.rs/ezmenu) crate.

mod struct_field;
mod struct_impl;

extern crate proc_macro;

use crate::struct_field::{FieldFormatting, FieldMenuInit};
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Attribute, Data, DataStruct, DeriveInput, Fields, FieldsNamed, Ident, Meta,
    Path,
};

macro_rules! run {
    (nested: $id:ident, $var:ident, $nested:expr, $s:expr) => {
        if let NestedMeta::Lit(Lit::$var(lit)) = $nested {
            $id = Some(lit.clone());
        } else {
            abort_invalid_type($nested, $s);
        }
    };

    ($id:ident, $var:ident, $lit:expr, $s:expr) => {
        if let Lit::$var(lit) = $lit {
            $id = Some(lit.clone());
        } else {
            abort_invalid_type($lit, $s);
        }
    };
}

use crate::struct_impl::MenuInit;
pub(crate) use run;

#[proc_macro_derive(Menu, attributes(menu))]
#[proc_macro_error]
pub fn build_menu(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(ts as DeriveInput);
    match input.data {
        Data::Enum(_e) => todo!("derive on enum soon"),
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => build_struct(input.ident, input.attrs, fields),
        _ => abort_call_site!("Menu derive supports only non-tuple structs and enums."),
    }
    .into()
}

#[cfg(feature = "custom_io")]
fn def_init<'a>(menu_desc: MenuInit) -> TokenStream {
    let fields = menu_desc.fields.iter().map(|field| &field.kind);
    quote! {
        pub fn from_io<R, W>(reader: R, writer: W) -> ::ezmenu::MenuResult<Self>
        where
            R: ::std::io::BufRead,
            W: ::std::io::Write,
        {
            let mut menu = ::ezmenu::StructMenu::new(reader, writer)
                #menu_desc;
            Ok(Self {#(
                #fields
            )*})
        }
    }
}

#[cfg(not(any(feature = "custom_io", test)))]
fn def_init<'a>(menu_desc: MenuInit) -> TokenStream {
    let fields = menu_desc.fields.iter().map(|field| &field.kind);
    quote! {
        pub fn from_menu() -> ::ezmenu::MenuResult<Self> {
            let mut menu = ::ezmenu::StructMenu::default()
                #menu_desc;
            Ok(Self {#(
                #fields
            )*})
        }
    }
}

#[inline(never)]
fn abort_invalid_type(span: impl ToTokens, s: &str) -> ! {
    abort!(
        span,
        "invalid literal type for `{}` attribute", s;
        help = "try surrounding: `{}(\"...\")`", s
    )
}

#[inline(never)]
fn abort_invalid_arg_name(span: impl ToTokens, s: &str) -> ! {
    abort!(span, "invalid argument name: `{}`", s)
}

#[inline]
fn path_to_string(from: &Path) -> String {
    // meta attribute parsing makes path always start with an ident
    from.get_ident().unwrap().to_string()
}

fn get_meta_attr(attrs: Vec<Attribute>) -> Option<Meta> {
    attrs.into_iter().find_map(|attr| {
        if attr.path.is_ident("menu") {
            let meta = attr
                .parse_meta()
                .unwrap_or_else(|e| abort!(attr, "incorrect definition of menu attribute: {}", e));
            Some(meta)
        } else {
            None
        }
    })
}

fn build_struct(name: Ident, attrs: Vec<Attribute>, fields: FieldsNamed) -> TokenStream {
    // optional menu attr of the struct
    let struct_attr = get_meta_attr(attrs);
    // fields of the struct mapped to menu fields description
    let fields = fields
        .named
        .into_iter()
        .map(|field| FieldMenuInit::from(field))
        .collect();

    let init = def_init(MenuInit::new(struct_attr, fields));

    quote! {
        impl #name {
            #init
        }
    }
}

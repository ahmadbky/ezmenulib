//! This crate is a derive procedural macro for `EZMenu` crate.
//! It should not be used directly. You must use the [`ezmenu`](https://docs.rs/ezmenu) crate.

mod struct_field;
mod struct_impl;

extern crate proc_macro as pm;

use crate::struct_field::{FieldFormatting, FieldMenuInit};
use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed,
    Ident, Meta, Path,
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

// TODO: parser attribute
#[proc_macro_attribute]
#[proc_macro_error]
pub fn parser(_attr: pm::TokenStream, _ts: pm::TokenStream) -> pm::TokenStream {
    pm::TokenStream::new()
}

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

        pub fn from_menu_unwrap() -> Self {
            Self::from_menu()
                .expect("An error occurred while processing menu")
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
        attr.path.is_ident("menu").then(|| {
            attr.parse_meta()
                .unwrap_or_else(|e| abort!(attr, "incorrect definition of menu attribute: {}", e))
        })
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

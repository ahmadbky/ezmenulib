//! Module used to generate code used to pretend that fields or variants of the input
//! are used, to avoid rustc warnings of unused fields or variants.
//!
//! This module is based on the serde_derive::pretend module. See its [documentation]
//! for more details on how this works.
//!
//! [documentation]: https://docs.rs/serde_derive/latest/src/serde_derive/pretend.rs.html

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, Token, Variant,
};

use crate::utils::{get_attr_with_args, get_lib_root, take_val};

/// Returns the code that is pretending to use the fields or variants of the input code,
/// to suppress the useless rustc warnings.
///
/// See the [module documentation](crate::pretend) for more information.
pub(crate) fn pretend_used(input: &DeriveInput) -> TokenStream {
    match &input.data {
        Data::Enum(DataEnum { variants, .. }) => used_enum(input, variants),
        Data::Struct(DataStruct { fields, .. }) => used_struct(input, fields),
        _ => quote!(),
    }
}

/// Returns the code that is pretending to use the variants of the input enum.
fn used_enum(input: &DeriveInput, variants: &Punctuated<Variant, Token![,]>) -> TokenStream {
    let name = &input.ident;
    let ty_gens = input.generics.split_for_impl().1;
    let turbofish = ty_gens.as_turbofish();

    let patterns: Punctuated<_, Token![,]> = variants
        .iter()
        .filter_map(|var| match &var.fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let var = &var.ident;
                let fields = named.iter().map(|f| &f.ident);
                let vals = (0..fields.len()).map(|i| format_ident!("__v{}", i));
                Some(quote!(#name::#var { #(#fields: #vals),* }))
            }
            _ => None,
        })
        .collect();

    let case = variants.iter().map(|var| {
        let var_name = &var.ident;
        let vals = &(0..var.fields.len())
            .map(|i| format_ident!("__v{}", i))
            .collect::<Vec<_>>();

        let pat = match &var.fields {
            Fields::Unit => quote!(),
            Fields::Named(FieldsNamed { named, .. }) => {
                let fields = named.iter().map(|f| &f.ident);
                quote!({ #(#fields: #vals),* })
            }
            Fields::Unnamed(_) => quote!((#(#vals),*)),
        };

        quote! {
            match ::core::option::Option::None {
                ::core::option::Option::Some((#(#vals,),*)) => {
                    let _ = #name::#var_name #turbofish #pat;
                }
                _ => (),
            }
        }
    });

    quote! {
        match ::core::option::Option::<&#name #ty_gens>::None {
            #patterns
            _ => (),
        }

        #(#case)*
    }
}

mod kw {
    use syn::custom_keyword;
    custom_keyword!(packed);
}

/// Struct used to represent the `#[packed]` attribute on a struct (I am not sure
/// of the utility of this but we never know, serde checks for it so 🤷‍♂️).
#[derive(Default)]
struct Packed(bool);

impl Parse for Packed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        while !input.is_empty() {
            if input.peek(kw::packed) {
                return Ok(Self(true));
            }
            input.parse::<Option<Token![,]>>()?;
        }
        Ok(Self(false))
    }
}

/// Returns the code that is pretending to use the fields of the input struct.
/// 
/// It checks if the struct is declared as packed or not.
fn used_struct(input: &DeriveInput, fields: &Fields) -> TokenStream {
    if !matches!(fields, Fields::Named(_)) {
        return TokenStream::new();
    }

    if let Some(Packed(true)) = get_attr_with_args(&input.attrs, "repr").map(take_val) {
        used_packed_struct(input, fields)
    } else {
        used_not_packed_struct(input, fields)
    }
}

/// Returns the code that is pretending to use the fields of a packed struct.
fn used_packed_struct(input: &DeriveInput, fields: &Fields) -> TokenStream {
    let name = &input.ident;
    let ty_gens = input.generics.split_for_impl().1;
    let fields = fields.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let root = get_lib_root().1;

    quote! {
        match #root::__private::Option::<&#name #ty_gens>::None {
            #root::__private::Option::Some(__v @ #name { #(#fields: _),* }) => {#(
                let _ = #root::__private::addr_of!(__v.#fields);
            )*}
            _ => (),
        }
    }
}

/// Returns the code that is pretending to use the fields of a non-packed struct.
fn used_not_packed_struct(input: &DeriveInput, fields: &Fields) -> TokenStream {
    let name = &input.ident;
    let ty_gens = input.generics.split_for_impl().1;

    let vals = (0..fields.len()).map(|i| format_ident!("__v{}", i));
    let fields = fields.iter().map(|f| &f.ident);

    let root = get_lib_root().1;

    quote! {
        match #root::__private::Option::<&#name #ty_gens>::None {
            #root::__private::Option::Some(#name { #(#fields: #vals),* }) => (),
            _ => (),
        }
    }
}

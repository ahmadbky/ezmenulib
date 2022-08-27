use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, Token, Variant,
};

use crate::utils::{get_attr_with_args, take_val};

pub(crate) fn pretend_used(input: &DeriveInput) -> TokenStream {
    match &input.data {
        Data::Enum(DataEnum { variants, .. }) => used_enum(input, variants),
        Data::Struct(DataStruct { fields, .. }) => used_struct(input, fields),
        _ => quote!(),
    }
}

fn used_enum(input: &DeriveInput, variants: &Punctuated<Variant, Token![,]>) -> TokenStream {
    let name = &input.ident;
    let (_, ty_gens, _) = input.generics.split_for_impl();
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

fn used_packed_struct(input: &DeriveInput, fields: &Fields) -> TokenStream {
    let name = &input.ident;
    let (_, ty_gens, _) = input.generics.split_for_impl();
    let fields = fields.iter().map(|f| &f.ident).collect::<Vec<_>>();

    #[cfg(not(no_ptr_addr_of))]
    {
        quote! {
            match ::core::option::Option::<&#name #ty_gens>::None {
                ::core::option::Option::Some(__v @ #name { #(#fields: _),* }) => {#(
                    let _ = ::core::ptr::addr_of!(__v.#fields);
                )*}
                _ => (),
            }
        }
    }

    #[cfg(no_ptr_addr_of)]
    {
        let vals = (0..fields.len()).map(|i| format_ident!("__v{}", i));

        quote! {
            match ::core::option::Option::<#name #ty_gens>::None {
                ::core::option::Option::Some(#name { #(#fields: #vals),* }) = (),
                _ => (),
            }
        }
    }
}

fn used_not_packed_struct(input: &DeriveInput, fields: &Fields) -> TokenStream {
    let name = &input.ident;
    let (_, ty_gens, _) = input.generics.split_for_impl();

    let vals = (0..fields.len()).map(|i| format_ident!("__v{}", i));
    let fields = fields.iter().map(|f| &f.ident);

    quote! {
        match ::core::option::Option::<&#name #ty_gens>::None {
            ::core::option::Option::Some(#name { #(#fields: #vals),* }) => (),
            _ => (),
        }
    }
}

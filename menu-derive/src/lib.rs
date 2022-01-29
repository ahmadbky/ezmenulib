extern crate proc_macro;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, abort_call_site, emit_error, proc_macro_error};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed, LitStr, Path, Token,
};

#[proc_macro_derive(Menu, attributes(field))]
#[proc_macro_error]
pub fn build_menu(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(ts as DeriveInput);
    let name = input.ident;

    match input.data {
        Data::Enum(_e) => todo!("derive on enum soon"),
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => build_struct(name, fields),
        _ => abort_call_site!("Menu derive supports only non-tuple structs and enums."),
    }
    .into()
}

struct MenuFieldDesc {
    msg: LitStr,
    then: Path,
}

impl Parse for MenuFieldDesc {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let msg = input.parse::<LitStr>()?;
        input.parse::<Token![,]>()?;
        input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let then = input.parse::<Path>()?;
        Ok(Self { msg, then })
    }
}

fn build_struct(name: Ident, fields: FieldsNamed) -> TokenStream {
    let fields = fields.named;

    let fields_desc = fields
        .iter()
        .map(|f| &f.attrs)
        .filter(|attrs| attrs.iter().any(|a| a.path.segments[0].ident == "field"))
        .map(|attrs| {
            attrs[0]
                .clone()
                .parse_args::<MenuFieldDesc>()
                .expect("Invalid field attribute")
        })
        .collect::<Vec<MenuFieldDesc>>();

    let f_ident = fields.iter().map(|f| f.ident.as_ref().unwrap());
    let f_inner = f_ident.clone();
    let f_type = fields.iter().map(|f| &f.ty);

    let f_msg = fields_desc.iter().map(|fd| &fd.msg);
    let f_then = fields_desc.iter().map(|fd| &fd.then);

    quote! {
        impl ::menu::Menu for #name {
            fn from_menu() -> Self {
                let stdin = ::std::io::stdin();
                let mut stdout = ::std::io::stdout();

                #(let #f_ident = ::menu::ask::<#f_type, _>(
                    &stdin,
                    &mut stdout,
                    #f_msg,
                    #f_then,
                );)*

                Self { #(#f_inner),* }
            }
        }
    }
}

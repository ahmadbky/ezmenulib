extern crate proc_macro;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed, Lit, LitStr, Meta,
    MetaList, MetaNameValue, NestedMeta, Path, Type,
};

#[proc_macro_derive(Menu, attributes(field, all))]
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

/// Returns the string contained in the first path segment
/// For example: `field::then(...)` returns `"field"`
fn first_seg_val(path: &Path) -> String {
    path.segments
        .first()
        .expect("expected path")
        .ident
        .to_string()
}

/// Struct used to contain information on a menu field
/// It contains all optional values because values
/// or the whole attribute can be omitted
#[derive(Default)]
struct FieldDesc {
    msg: Option<LitStr>,
    then: Option<Path>,
}

// TODO: better error handling
impl From<Meta> for FieldDesc {
    fn from(m: Meta) -> Self {
        dbg!("{:#?}", &m);
        // values modified at each iteration
        // (if the user provided them multiple times)
        let mut msg = None;
        let mut then = None;
        // root meta must be a list of metas
        if let Meta::List(MetaList { nested, .. }) = m {
            for nm in nested {
                match nm {
                    // in inner metas, if the meta type is a list,
                    // then it should contain only 1 nested meta as value
                    // like a path to a function, or a string literal for a message
                    NestedMeta::Meta(Meta::List(MetaList { path, nested, .. })) => {
                        // get the first nested meta inside parenthesis
                        let nested = nested.first().expect("value missing");
                        match first_seg_val(&path).as_str() {
                            "msg" => if let NestedMeta::Lit(Lit::Str(lit)) = nested {
                                msg = Some(lit.clone());
                            } else {
                                abort_call_site!("string literal value expected for `msg` attribute");
                            },
                            "then" => if let NestedMeta::Meta(Meta::Path(path)) = nested {
                                then = Some(path.clone());
                            } else {
                                abort_call_site!("path to function expected for `then` attribute");
                            }
                            s => abort_call_site!("incorrect name: `{}`", s),
                        }
                    }
                    // deconstructing to a path and a literal
                    // here i don't check in the pattern of the lit type is a string literal
                    // for future features maybe
                    NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) => {
                        match first_seg_val(&path).as_str() {
                            "msg" => if let Lit::Str(l_str) = lit {
                                msg = Some(l_str);
                            } else {
                                abort_call_site!("invalid literal type for `msg` attribute");
                            },
                            s => abort_call_site!("incorrect name: `{}`", s),
                        }
                    }
                    _ => abort_call_site!("identifier - value attributes must be formatted like this: msg(\"literal\") or msg = \"literal\""),
                }
            }
        } else {
            abort_call_site!("incorrect definition of field attribute");
        }
        Self { msg, then }
    }
}

fn build_struct(name: Ident, fields: FieldsNamed) -> TokenStream {
    let fields = fields.named;

    // menu field description of each struct field
    let fields_desc = fields
        .iter()
        .map(|f| {
            f.attrs
                .iter()
                .find(|attr| first_seg_val(&attr.path) == "field")
                .cloned()
                .map(|attr| {
                    attr.parse_meta()
                        .expect("incorrect definition of field attribute")
                        .into()
                })
                .unwrap_or(FieldDesc::default())
        })
        .collect::<Vec<FieldDesc>>();

    let f_ident = fields.iter().map(|f| f.ident.as_ref().unwrap());
    let f_inner = f_ident.clone();
    let f_type = fields.iter().map(|f| &f.ty);

    let f_msg = fields_desc.iter().zip(f_ident.clone()).map(|(fd, ident)| {
        fd.msg
            .as_ref()
            .map(|lit| quote!(#lit))
            .unwrap_or(quote!(stringify!(#ident)))
    });

    let f_then = fields_desc.iter().map(|fd| {
        fd.then
            .as_ref()
            .map(|path| quote!(#path))
            .unwrap_or(quote!(|_| {}))
    });

    quote! {
        impl #name {
            fn from_menu() -> Self {
                let stdin = ::std::io::stdin();
                let mut stdout = ::std::io::stdout();

                #(let #f_ident: #f_type = ::ezmenu::ask(
                    &mut stdin.lock(),
                    &mut stdout,
                    #f_msg,
                    #f_then,
                );)*

                Self { #(#f_inner),* }
            }
        }
    }
}

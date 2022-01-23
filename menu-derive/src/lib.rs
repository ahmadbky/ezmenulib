extern crate proc_macro;

use proc_macro2::{Ident, Literal, TokenStream, TokenTree};
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, Fields,
    FieldsNamed, Index, LitStr, Path, Token,
};

#[proc_macro_derive(Menu, attributes(main, field))]
#[proc_macro_error]
pub fn build_menu(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(ts as DeriveInput);
    println!("{:#?}", input);
    let name = input.ident;

    match input.data {
        Data::Enum(e) => build_enum(name, e, &input.attrs),
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => build_struct(name, fields),
        _ => abort_call_site!("Menu macro works only on non-tuple structs and enums."),
    }
    .into()
}

#[derive(Debug)]
struct MenuDesc {
    msg: Option<LitStr>,
    //exec: Path,
}

impl Parse for MenuDesc {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // msg ident
        input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let msg = input.parse::<LitStr>().ok();
        //input.parse::<Token![,]>()?;
        // exec ident
        //input.parse::<Ident>()?;
        //let exec = input.parse::<Path>()?;
        Ok(Self { msg, /* exec */ })
    }
}

fn get_msg(v: &Vec<Attribute>) -> String {
    v.iter()
        .filter_map(|attr| {
            if attr.path.segments.len() > 0 && &attr.path.segments[0].ident == "menu" {
                Some(match attr.parse_args::<MenuDesc>() {
                    Ok(e) => Some(e),
                    Err(x) => {
                        println!("{:?}", x);
                        None
                    }
                })
            } else {
                None
            }
        })
        .filter_map(|md| {
            println!("{:?}", md);
            md.map(|m| m.msg.map(|s| s.value() + "\n")).flatten()
        })
        .collect()
}

fn get_executor(v: &Vec<Attribute>) -> Path {
    todo!()
}

fn build_enum(name: Ident, e: DataEnum, attrs: &Vec<Attribute>) -> TokenStream {
    let field_idents = e.variants.iter().map(|v| &v.ident);
    let funcs_exec = e.variants.iter().map(|v| get_executor(&v.attrs));
    let range = (1..=e.variants.len()).map(|i| Index::from(i));

    let main_msg = get_msg(attrs);
    quote! {
        impl #name {
            pub fn run() {
                use std::io::{stdin, stdout, Write};
                let mut stdout = stdout();
                let stdin = stdin();
                print!("{}", #main_msg);

                let msgs = [#(stringify!(#field_idents)),*];
                for (i, msg) in msgs.iter().enumerate() {
                    println!("{} - {}", i+1, msg);
                }

                let i = loop {
                    print!(">> ");
                    stdout.flush().expect("Unable to flush stdout");
                    let mut buf = String::new();
                    stdin.read_line(&mut buf).expect("Unable to read line");
                    match buf.trim().parse::<usize>() {
                        Ok(x) if (1..=msgs.len()).contains(&x) => break x,
                        _ => continue,
                    };
                };

                match i {
                    #(
                        #range => println!("wooohooo: {}", #range),
                    )*
                    _ => unreachable!(),
                }
            }
        }
    }
}

fn build_struct(name: Ident, _fields: FieldsNamed) -> TokenStream {
    quote! {
        impl #name {
            fn run() -> Self {
                todo!()
            }
        }
    }
}

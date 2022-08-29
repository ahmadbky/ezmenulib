use proc_macro2::{Delimiter, Group, Span, TokenStream};
use proc_macro_error::{abort, abort_call_site, set_dummy};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    punctuated::Punctuated, Attribute, Data, DataEnum, DeriveInput, Expr, Fields, Ident, Index,
    LitStr, Path, Token, Variant,
};

use crate::{
    format::Format,
    kw::define_attr,
    pretend::pretend_used,
    prompted::FunctionExpr,
    utils::{
        get_attr_with_args, get_first_doc, get_lib_root, method_call, method_call_empty,
        split_ident_camel_case, wrap_in_const, Case,
    },
};

define_attr! {
    RawEntryAttr {
        msg: Option<LitStr>,
        case: Option<Case>,
        raw: bool; without msg,
        nodoc: bool; without msg,

        mapped: Option<(Path, Punctuated<Expr, Token![,]>)>; without
            map; parent; back; quit,
        map: Option<FunctionExpr>; without mapped; parent; back; quit,
        parent: bool; without mapped; map; back; quit,
        back: Option<Index>; without mapped; map; parent; quit,
        quit: bool; without mapped; map; parent; back,
    }
}

enum EntryKind {
    Mapped(Path, Punctuated<Expr, Token![,]>),
    Map(FunctionExpr),
    Parent(Ident),
    Back(Index),
    Quit,
}

impl EntryKind {
    fn new(id: &Ident, raw_attr: &RawEntryAttr) -> Self {
        if let Some((path, args)) = &raw_attr.mapped {
            Self::Mapped(path.clone(), args.clone())
        } else if let Some(func) = &raw_attr.map {
            Self::Map(func.clone())
        } else if let Some(i) = &raw_attr.back {
            Self::Back(i.clone())
        } else if raw_attr.parent {
            Self::Parent(id.clone())
        } else {
            Self::Quit
        }
    }
}

impl ToTokens for EntryKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root().1;
        quote!(#root::field::kinds::).to_tokens(tokens);

        let (id, args) = match self {
            Self::Mapped(path, args) => ("map", quote!(move |s| #path(s, #args))),
            Self::Map(func) => ("map", func.to_token_stream()),
            Self::Parent(id) => ("parent", quote!(<#id as #root::menu::Menu>::fields())),
            Self::Back(i) => ("back", i.to_token_stream()),
            Self::Quit => ("quit", TokenStream::new()),
        };

        tokens.append(Ident::new(id, Span::call_site()));
        tokens.append(Group::new(Delimiter::Parenthesis, args));
    }
}

struct EntryField {
    msg: String,
    kind: EntryKind,
}

impl EntryField {
    fn new(var: Variant, global_case: Option<Case>) -> Self {
        match &var.fields {
            Fields::Unit => (),
            other => {
                let fields = match other {
                    Fields::Named(named) => named.into_token_stream(),
                    Fields::Unnamed(unnamed) => unnamed.into_token_stream(),
                    Fields::Unit => unreachable!(),
                }
                .to_string();
                abort!(
                    var, "derive(Menu) supports only unit enums";
                    help = "you might want to remove these fields: `{}`", fields,
                );
            }
        }

        let attr = get_attr_with_args(&var.attrs, "menu")
            .unwrap_or_default()
            .val;

        let kind = EntryKind::new(&var.ident, &attr);

        let RawEntryAttr {
            msg,
            case,
            raw,
            nodoc,
            ..
        } = attr;

        let msg = msg
            .map(|l| l.value())
            .or_else(|| {
                if nodoc {
                    None
                } else {
                    get_first_doc(&var.attrs)
                }
            })
            .unwrap_or_else(|| {
                if raw {
                    var.ident.to_string()
                } else {
                    split_ident_camel_case(&var.ident)
                }
            });
        let msg = case.or(global_case).unwrap_or_default().map(msg);

        Self { msg, kind }
    }
}

impl ToTokens for EntryField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let msg = &self.msg;
        let kind = &self.kind;
        quote!((#msg, #kind)).to_tokens(tokens);
    }
}

define_attr! {
    RootAttr {
        title: Option<LitStr>,
        raw: bool,
        no_title: bool; without title; raw,
        nodoc: bool; without title,
        fmt: Option<Format>,
        case: Option<Case>,
        once: bool,
    }
}

struct RootData {
    title: Option<String>,
    case: Option<Case>,
    fmt: Option<Format>,
    once: bool,
}

impl RootData {
    fn new(name: &Ident, attrs: &[Attribute]) -> Self {
        let RootAttr {
            title,
            raw,
            no_title,
            nodoc,
            fmt,
            case,
            once,
        } = get_attr_with_args(attrs, "menu").unwrap_or_default().val;

        let title = if no_title {
            None
        } else {
            Some(
                title
                    .map(|l| l.value())
                    .or_else(|| if nodoc { None } else { get_first_doc(attrs) })
                    .unwrap_or_else(|| {
                        if raw {
                            name.to_string()
                        } else {
                            split_ident_camel_case(name)
                        }
                    }),
            )
        };

        Self {
            title,
            case,
            fmt,
            once,
        }
    }
}

pub(crate) fn build_menu(input: DeriveInput) -> TokenStream {
    let used = pretend_used(&input);

    let name = input.ident;
    let root = get_lib_root().1;

    set_dummy(wrap_in_const(quote! {
        impl #root::menu::Menu for #name {
            fn fields<'a, __H: #root::menu::Handle + 'static>() -> #root::field::Fields<'a, __H> {
                #used
                unimplemented!()
            }

            fn raw_menu<'a, __H: #root::menu::Handle + 'static>(
                _: __H,
            ) -> #root::menu::RawMenu<'a, __H> {
                unimplemented!()
            }
        }
    }));

    let data = RootData::new(&name, &input.attrs);

    let fmt_fn = data.fmt.map(|f| method_call("format", f));
    let title_fn = method_call("title", data.title);
    let once_fn = data.once.then(|| method_call_empty("once"));

    let fields = match input.data {
        Data::Enum(DataEnum { variants, .. }) => {
            variants.into_iter().map(|v| EntryField::new(v, data.case))
        }
        _ => abort_call_site!("derive(Menu) supports only unit enums"),
    };

    wrap_in_const(quote! {
        impl #root::menu::Menu for #name {
            fn fields<'a, __H: #root::menu::Handle + 'static>() -> #root::field::Fields<'a, __H> {
                #used
                #root::__private::vec![#(#fields),*]
            }

            fn raw_menu<'a, __H: #root::menu::Handle + 'static>(
                __h: __H,
            ) -> #root::menu::RawMenu<'a, __H> {
                #root::menu::RawMenu::with_handle(__h, Self::fields())
                #fmt_fn
                #title_fn
                #once_fn
            }
        }
    })
}

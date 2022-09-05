use proc_macro2::{Delimiter, Group, Span, TokenStream};
use proc_macro_error::{abort, abort_call_site, set_dummy};
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Data, DataEnum, DeriveInput, Expr, ExprClosure, Fields, Ident, Index, LitStr, Pat,
    PatIdent, Path, Token, Variant,
};

use crate::{
    format::Format,
    kw::define_attr,
    pretend::pretend_used,
    prompted::FunctionExpr,
    utils::{
        get_attr_with_args, get_first_doc, get_last_seg_of_path, get_lib_root, method_call,
        method_call_empty, split_ident_camel_case, wrap_in_const, Case,
    },
};

define_attr! {
    RawEntryAttr {
        msg: Option<LitStr>,
        case: Option<Case>,
        raw: bool; without msg,
        nodoc: bool; without msg,

        flatten: bool; without mapped_with; mapped; map_with; map; parent; back; quit,
        mapped_with: Option<MappedWith>; without mapped; map_with; map; parent; back; quit;
            flatten,
        mapped: Option<(Path, Punctuated<Expr, Token![,]>)>; without
            mapped_with; map_with; map; parent; back; quit; flatten,
        map_with: Option<MapWith>; without mapped_with; mapped; map; parent; back; quit; flatten,
        map: Option<FunctionExpr>; without mapped_with; mapped; map_with; parent; back; quit; flatten,
        parent: bool; without mapped_with; mapped; map_with; map; back; quit; flatten,
        back: Option<Index>; without mapped_with; mapped; map_with; map; parent; quit; flatten,
        quit: bool; without mapped_with; mapped; map_with; map; parent; back; flatten,
    }
}

#[derive(Debug, Clone)]
struct InnerMapWith {
    mutable: bool,
    static_path: Path,
    static_ident: Ident,
}

impl Parse for InnerMapWith {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mutable = if input.peek(Token![mut]) {
            input.parse::<Token![mut]>()?;
            true
        } else {
            false
        };

        let static_path = input.parse()?;
        let static_ident = get_last_seg_of_path(&static_path).unwrap().ident.clone();
        let static_ident = Ident::new(
            static_ident.to_string().to_lowercase().as_str(),
            static_ident.span(),
        );
        input.parse::<Token![:]>()?;

        Ok(Self {
            mutable,
            static_path,
            static_ident,
        })
    }
}

impl InnerMapWith {
    fn to_tokens_with(&self, tokens: &mut TokenStream, arg: TokenStream) {
        let root = get_lib_root().1;
        let static_path = &self.static_path;
        let span = static_path.span();
        let map_fn = Ident::new(if self.mutable { "map_mut" } else { "map" }, span);

        quote_spanned! {span=>
            |__h| #root::__private::MutableStatic::#map_fn(
                &#static_path, __h, #arg
            )
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MappedWith {
    inner: InnerMapWith,
    fn_path: Path,
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for MappedWith {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let inner = input.call(Parse::parse)?;
        let fn_path = input.parse()?;
        let args = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            input.parse_terminated(Parse::parse)?
        } else {
            Punctuated::new()
        };

        Ok(Self {
            inner,
            fn_path,
            args,
        })
    }
}

impl ToTokens for MappedWith {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let static_ident = &self.inner.static_ident;
        let fn_path = &self.fn_path;
        let args = &self.args;

        self.inner.to_tokens_with(
            tokens,
            quote!(move |__h, #static_ident| #fn_path(__h, #static_ident, #args)),
        );
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MapWith {
    inner: InnerMapWith,
    fn_expr: FunctionExpr,
}

impl Parse for MapWith {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let inner = input.call(InnerMapWith::parse)?;

        let mut fn_expr = input.parse::<FunctionExpr>()?;
        if let FunctionExpr::Closure(ExprClosure { inputs, .. }) = &mut fn_expr {
            inputs.push(Pat::Ident(PatIdent {
                ident: inner.static_ident.clone(),
                attrs: vec![],
                by_ref: None,
                mutability: None,
                subpat: None,
            }));
        }

        Ok(Self { inner, fn_expr })
    }
}

impl ToTokens for MapWith {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.inner
            .to_tokens_with(tokens, self.fn_expr.to_token_stream());
    }
}

enum EntryKindType<'a> {
    MappedWith(MappedWith),
    Mapped(Path, Punctuated<Expr, Token![,]>),
    MapWith(MapWith),
    Map(FunctionExpr),
    Parent(Ident, &'a TokenStream),
    Back(Index),
    Quit,
}

impl ToTokens for EntryKindType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (id, args) = match &self {
            Self::MappedWith(mapped) => ("map", mapped.to_token_stream()),
            Self::Mapped(path, args) => ("map", quote!(move |__h| #path(__h, #args))),
            Self::MapWith(map) => ("map", map.to_token_stream()),
            Self::Map(map) => ("map", map.to_token_stream()),
            Self::Parent(id, trait_path) => (
                "parent",
                quote_spanned!(id.span()=> <#id as #trait_path>::fields()),
            ),
            Self::Back(i) => ("back", i.to_token_stream()),
            Self::Quit => ("quit", TokenStream::new()),
        };

        tokens.append(Ident::new(id, Span::call_site()));
        tokens.append(Group::new(Delimiter::Parenthesis, args));
    }
}

struct EntryKind<'a> {
    fields_path: &'a TokenStream,
    ty: EntryKindType<'a>,
}

impl ToTokens for EntryKind<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.fields_path.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

enum EntryField<'a> {
    Flattened {
        trait_path: &'a TokenStream,
        name: Ident,
    },
    Regular {
        msg: String,
        kind: Box<EntryKind<'a>>,
    },
}

impl<'a> EntryField<'a> {
    fn new(
        var: Variant,
        global_case: Option<Case>,
        fields_path: &'a TokenStream,
        trait_path: &'a TokenStream,
    ) -> Self {
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

        let attr: RawEntryAttr = get_attr_with_args(&var.attrs, "menu")
            .unwrap_or_default()
            .val;

        if attr.flatten {
            Self::Flattened {
                trait_path,
                name: var.ident,
            }
        } else {
            let ty = if let Some((path, args)) = attr.mapped {
                EntryKindType::Mapped(path, args)
            } else if let Some(map) = attr.map {
                EntryKindType::Map(map)
            } else if let Some(i) = attr.back {
                EntryKindType::Back(i)
            } else if let Some(map) = attr.map_with {
                EntryKindType::MapWith(map)
            } else if let Some(map) = attr.mapped_with {
                EntryKindType::MappedWith(map)
            } else if attr.parent {
                EntryKindType::Parent(var.ident.clone(), trait_path)
            } else {
                EntryKindType::Quit
            };

            let kind = Box::new(EntryKind { fields_path, ty });

            let msg = attr
                .msg
                .map(|l| l.value())
                .or_else(|| {
                    if attr.nodoc {
                        None
                    } else {
                        get_first_doc(&var.attrs)
                    }
                })
                .unwrap_or_else(|| {
                    if attr.raw {
                        var.ident.to_string()
                    } else {
                        split_ident_camel_case(&var.ident)
                    }
                });
            let msg = attr.case.or(global_case).unwrap_or_default().map(msg);

            Self::Regular { msg, kind }
        }
    }
}

impl ToTokens for EntryField<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root().1;
        method_call(
            "chain",
            match self {
                EntryField::Flattened { trait_path, name } => {
                    quote_spanned!(name.span()=> <#name as #trait_path>::fields())
                }
                Self::Regular { msg, kind } => {
                    quote!(#root::__private::vec![(#msg, #kind)])
                }
            },
        )
        .to_tokens(tokens);
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
        tui: bool; without once,
    }
}

struct RootData {
    title: Option<String>,
    case: Option<Case>,
    fmt: Option<Format>,
    once: bool,
    tui: bool,
    fields_path: TokenStream,
    trait_path: TokenStream,
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
            tui,
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

        let root = get_lib_root().1;
        let (fields_path, trait_path) = if tui {
            (quote!(#root::tui::), quote!(#root::tui::Menu))
        } else {
            (quote!(#root::field::kinds::), quote!(#root::menu::Menu))
        };

        Self {
            title,
            case,
            fmt,
            once,
            tui,
            fields_path,
            trait_path,
        }
    }

    fn quote_with(
        &self,
        name: &Ident,
        fields_ts: TokenStream,
        menu_ts: TokenStream,
    ) -> TokenStream {
        let root = get_lib_root().1;

        let out = if self.tui {
            quote! {
                impl #root::tui::Menu for #name {
                    fn fields<'a, __B: #root::__private::tui::backend::Backend + #root::__private::Write + 'static>(
                    ) -> #root::tui::TuiFields<'a, __B> {
                        #fields_ts
                    }

                    fn tui_menu<'a, __B: #root::__private::tui::backend::Backend + #root::__private::Write + 'static>(
                    ) -> #root::tui::TuiMenu<'a, __B> {
                        #menu_ts
                    }
                }
            }
        } else {
            quote! {
                impl #root::menu::Menu for #name {
                    fn fields<'a, __H: #root::menu::Handle + 'static>() -> #root::field::Fields<'a, __H> {
                        #fields_ts
                    }

                    fn raw_menu<'a, __H: #root::menu::Handle + 'static>(__h: __H) -> #root::menu::RawMenu<'a, __H> {
                        #menu_ts
                    }
                }
            }
        };

        wrap_in_const(out)
    }

    fn dummy_token_stream_with(&self, name: &Ident, used: &TokenStream) -> TokenStream {
        self.quote_with(
            name,
            quote!(#used unimplemented!()),
            quote!(unimplemented!()),
        )
    }

    fn to_token_stream_with(
        &self,
        name: &Ident,
        used: &TokenStream,
        fields: Vec<EntryField<'_>>,
    ) -> TokenStream {
        let root = get_lib_root().1;

        let fmt_fn = self.fmt.as_ref().map(|f| method_call("format", f));

        let menu_ts = if self.tui {
            let title_fn = self.title.as_ref().map(|title| {
                method_call(
                    "block",
                    quote! {
                        #root::__private::tui::widgets::Block::default()
                            .title(#title)
                            .borders(#root::__private::tui::widgets::Borders::all())
                    },
                )
            });

            quote! {
                #root::tui::TuiMenu::new(Self::fields())
                #fmt_fn
                #title_fn
            }
        } else {
            let title_fn = self.title.as_ref().map(|t| method_call("title", t));
            let once_fn = self.once.then(|| method_call_empty("once"));
            quote! {
                #root::menu::RawMenu::with_handle(__h, Self::fields())
                #fmt_fn
                #title_fn
                #once_fn
            }
        };

        self.quote_with(
            name,
            quote!(#used [].into_iter() #(#fields)* .collect()),
            menu_ts,
        )
    }
}

pub(crate) fn build_menu(input: DeriveInput) -> TokenStream {
    let used = pretend_used(&input);
    let name = input.ident;
    let data = RootData::new(&name, &input.attrs);

    set_dummy(data.dummy_token_stream_with(&name, &used));

    let fields = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants
            .into_iter()
            .map(|v| EntryField::new(v, data.case, &data.fields_path, &data.trait_path))
            .collect(),
        _ => abort_call_site!("derive(Menu) supports only unit enums"),
    };

    data.to_token_stream_with(&name, &used, fields)
}

use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Index, LitStr, Token,
};

use crate::{
    format::Format,
    kw,
    utils::{get_lib_root, method_call, MethodCall},
};

use super::FunctionExpr;

#[derive(Clone, Debug)]
struct SelectedField {
    lit: LitStr,
    val: Expr,
}

impl ToTokens for SelectedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let lit = &self.lit;
        let val = &self.val;
        tokens.extend(quote!((#lit, #val)));
    }
}

impl Parse for SelectedField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit = input.parse()?;
        input.parse::<Token![,]>()?;
        let val = input.parse()?;

        Ok(Self { lit, val })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RawSelectedField {
    raw: SelectedField,
    default: Option<Span>,
}

impl Parse for RawSelectedField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let default = if input.peek(kw::default) {
            Some(input.parse::<kw::default>()?.span)
        } else {
            None
        };

        let content;
        parenthesized!(content in input);
        let raw = content.parse()?;

        Ok(Self { raw, default })
    }
}

fn get_default_fn<I: Iterator<Item = RawSelectedField>>(input: I) -> Option<MethodCall<Index>> {
    let mut default = None;

    for (i, v) in input.enumerate() {
        if let Some(span) = v.default {
            if default.is_none() {
                default = Some(Index::from(i));
            } else {
                abort!(span, "there is already a default selected field defined");
            }
        }
    }

    default.map(|i| method_call("default", i))
}

pub(crate) struct Selected {
    msg: String,
    entries: Punctuated<SelectedField, Token![,]>,
    fmt: Option<MethodCall<Format>>,
    default: Option<MethodCall<Index>>,
}

impl Selected {
    pub(crate) fn new(
        msg: String,
        fmt: Option<MethodCall<Format>>,
        entries: Punctuated<RawSelectedField, Token![,]>,
    ) -> syn::Result<Self> {
        let entries = entries.into_iter();
        let default = get_default_fn(entries.clone());
        let entries = entries.map(|raw| /* IM A BEAR */ raw.raw).collect();

        Ok(Self {
            msg,
            entries,
            fmt,
            default,
        })
    }
}

impl ToTokens for Selected {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root();
        let msg = &self.msg;
        let entries = &self.entries;

        let mut out = quote!(#root::field::Selected(#msg, [#entries]));
        self.fmt.to_tokens(&mut out);
        self.default.to_tokens(&mut out);

        tokens.extend(out);
    }
}

pub(crate) struct Written {
    pub(crate) msg: String,
    pub(crate) fmt: Option<MethodCall<Format>>,
    pub(crate) example: Option<MethodCall<LitStr>>,
    pub(crate) default_val: Option<MethodCall<LitStr>>,
    pub(crate) default_env: Option<MethodCall<LitStr>>,
}

impl ToTokens for Written {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root();
        let msg = &self.msg;

        let mut out = quote!(#root::field::Written::new(#msg));
        self.fmt.to_tokens(&mut out);
        self.example.to_tokens(&mut out);
        self.default_val.to_tokens(&mut out);
        self.default_env.to_tokens(&mut out);

        tokens.extend(out);
    }
}

pub(crate) struct WrittenUntil {
    pub(crate) w: Written,
    pub(crate) til: FunctionExpr,
}

impl ToTokens for WrittenUntil {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root();
        let w = &self.w;
        let til = &self.til;
        quote!(#root::field::WrittenUntil::from_written(#w, #til)).to_tokens(tokens);
    }
}

pub(crate) struct Separated {
    pub(crate) w: Written,
    pub(crate) sep: LitStr,
    pub(crate) env_sep: Option<MethodCall<TokenStream>>,
}

impl ToTokens for Separated {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root();
        let w = &self.w;
        let sep = &self.sep;
        quote!(#root::field::Separated::from_written(#w, #sep)).to_tokens(tokens);
        self.env_sep.to_tokens(tokens);
    }
}

pub(crate) struct Bool {
    pub(crate) w: Written,
}

impl ToTokens for Bool {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root();
        let w = &self.w;
        quote!(#root::field::Bool::from_written(#w)).to_tokens(tokens);
    }
}

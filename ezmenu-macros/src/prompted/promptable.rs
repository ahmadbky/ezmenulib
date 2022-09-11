//! Contains the expansion types of the promptable types of the library.

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

use super::{FunctionExpr, Promptable};

/// Represents a selectable field for the Selected promptable.
///
/// A selectable field is expanded to a couple ("msg", bound_value)
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

/// Represents a selectable field given in an attribute.
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

/// Returns the `default` method call for the selected promptable.
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

/// Represents the Selected promptable construction expansion.
pub(super) struct Selected {
    msg: String,
    entries: Punctuated<SelectedField, Token![,]>,
    fmt: Option<MethodCall<Format>>,
    default: Option<MethodCall<Index>>,
}

impl Selected {
    /// Returns the Selected promptable expansion handle from the selectable field found in
    /// the prompted attribute.
    pub(super) fn new(
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
        let root = get_lib_root().1;
        let msg = &self.msg;
        let entries = &self.entries;

        let mut out = quote!(#root::field::Selected(#msg, [#entries]));
        self.fmt.to_tokens(&mut out);
        self.default.to_tokens(&mut out);

        tokens.extend(out);
    }
}

/// Represents the Written promptable construction expansion.
pub(super) struct Written {
    pub(super) msg: String,
    pub(super) fmt: Option<MethodCall<Format>>,
    pub(super) example: Option<MethodCall<LitStr>>,
    pub(super) or_val: Option<MethodCall<LitStr>>,
    pub(super) or_env: Option<MethodCall<LitStr>>,
}

impl ToTokens for Written {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root().1;
        let msg = &self.msg;

        let mut out = quote!(#root::field::Written::new(#msg));
        self.fmt.to_tokens(&mut out);
        self.example.to_tokens(&mut out);
        self.or_val.to_tokens(&mut out);
        self.or_env.to_tokens(&mut out);

        tokens.extend(out);
    }
}

/// Represents the Until promptable construction expansion.
pub(super) struct Until {
    pub(super) inner: Box<Promptable>,
    pub(super) til: FunctionExpr,
}

impl ToTokens for Until {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root().1;
        let inner = &self.inner;
        let til = &self.til;
        quote!(#root::field::Until::from_promptable(#inner, #til)).to_tokens(tokens);
    }
}

/// Represents the Separated promptable construction expansion.
pub(super) struct Separated {
    pub(super) w: Written,
    pub(super) sep: LitStr,
    pub(super) env_sep: Option<MethodCall<(LitStr, LitStr)>>,
}

impl ToTokens for Separated {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root().1;
        let w = &self.w;
        let sep = &self.sep;
        quote!(#root::field::Separated::from_written(#w, #sep)).to_tokens(tokens);
        self.env_sep.to_tokens(tokens);
    }
}

/// Represents the Bool promptable construction expansion.
pub(super) struct Bool {
    pub(super) w: Written,
    pub(super) basic_example: Option<MethodCall<()>>,
}

impl ToTokens for Bool {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root().1;
        let w = &self.w;
        quote!(#root::field::Bool::from_written(#w)).to_tokens(tokens);
        self.basic_example.to_tokens(tokens);
    }
}

/// Represents the Password promptable construction expansion.
pub(super) struct Password {
    pub(super) msg: String,
    pub(super) fmt: Option<MethodCall<Format>>,
}

impl ToTokens for Password {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root().1;
        let msg = &self.msg;
        quote!(#root::field::Password::new(#msg)).to_tokens(tokens);
        self.fmt.to_tokens(tokens);
    }
}

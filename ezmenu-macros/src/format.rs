use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitBool, LitStr, Token,
};

use crate::utils::{abort_invalid_ident, get_lib_root, to_str};

/// Represents a format parameter specified inside the `fmt(...)` meta attribute.
#[derive(Clone)]
enum Param {
    /// `prefix/pref: "..."`.
    Prefix(LitStr),
    /// `left/left_sur: "..."`.
    LeftSur(LitStr),
    /// `right/right_sur: "..."`.
    RightSur(LitStr),
    /// `chip: "..."`.
    Chip(LitStr),
    /// `show_default/show_d/default/d: "..."` or `no_default/no_d` alone,
    /// setting the bool at false.
    ShowDefault(bool),
    /// `suffix/suf: "..."`
    Suffix(LitStr),
    /// `line_brk/line_break/brk/break: "..."` or `no_break/no_brk` alone
    /// setting the bool at false.
    LineBrk(bool),
}

impl Parse for Param {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id = input.parse::<Ident>()?;
        Ok(match to_str!(id) {
            "no_default" | "no_d" => Self::ShowDefault(false),
            "default" | "d" | "show_default" | "show_d" => {
                let b = if input.peek(Token![=]) {
                    input.parse::<Token![:]>()?;
                    input.parse::<LitBool>()?.value()
                } else {
                    true
                };
                Self::ShowDefault(b)
            }
            "no_brk" | "no_break" => Self::LineBrk(false),
            "brk" | "break" | "line_brk" | "line_break" => {
                let b = if input.peek(Token![=]) {
                    input.parse::<Token![:]>()?;
                    input.parse::<LitBool>()?.value()
                } else {
                    true
                };
                Self::LineBrk(b)
            }
            other => {
                input.parse::<Token![:]>()?;
                let lit = input.parse()?;
                match other {
                    "prefix" | "pref" => Self::Prefix(lit),
                    "left_sur" | "left" => Self::LeftSur(lit),
                    "right_sur" | "right" => Self::RightSur(lit),
                    "chip" => Self::Chip(lit),
                    "suffix" | "suf" => Self::Suffix(lit),
                    _ => abort_invalid_ident(
                        id,
                        &[
                            "line_break",
                            "line_brk",
                            "show_default",
                            "show_d",
                            "no_default",
                            "no_d",
                            "default",
                            "d",
                            "no_brk",
                            "no_break",
                            "brk",
                            "break",
                            "prefix",
                            "pref",
                            "left_sur",
                            "left",
                            "right_sur",
                            "right",
                            "chip",
                            "suffix",
                            "suf",
                        ],
                    ),
                }
            }
        })
    }
}

/// Represents the `fmt(...)` meta attribute, with its custom parameters,
/// used to represent it as a `Format` struct construct.
pub struct Format {
    prefix: Option<LitStr>,
    left_sur: Option<LitStr>,
    right_sur: Option<LitStr>,
    chip: Option<LitStr>,
    show_default: Option<bool>,
    suffix: Option<LitStr>,
    line_brk: Option<bool>,

    // Set at `true` if not all the parameters have been provided so we must place the
    // `..Default::default()` line at the end of the Format struct instantiation.
    some_omitted: bool,
}

impl Parse for Format {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut prefix = None;
        let mut left_sur = None;
        let mut right_sur = None;
        let mut chip = None;
        let mut show_default = None;
        let mut suffix = None;
        let mut line_brk = None;

        let mut vals = Punctuated::<_, Token![,]>::parse_terminated(input)?.into_iter();

        for _ in 0..7.min(vals.len()) {
            match vals.next() {
                Some(Param::LineBrk(b)) => line_brk = Some(b),
                Some(Param::Suffix(l)) => suffix = Some(l),
                Some(Param::ShowDefault(b)) => show_default = Some(b),
                Some(Param::RightSur(l)) => right_sur = Some(l),
                Some(Param::Chip(l)) => chip = Some(l),
                Some(Param::LeftSur(l)) => left_sur = Some(l),
                Some(Param::Prefix(l)) => prefix = Some(l),
                None => (),
            }
        }

        let some_omitted = prefix.is_none()
            || left_sur.is_none()
            || right_sur.is_none()
            || chip.is_none()
            || show_default.is_none()
            || suffix.is_none()
            || line_brk.is_none();

        Ok(Self {
            prefix,
            left_sur,
            right_sur,
            chip,
            show_default,
            suffix,
            line_brk,
            some_omitted,
        })
    }
}

/// Internal macro used to map a format parameter to its `field: value,` token-stream
/// representation in the construction of the Format struct.
macro_rules! map_to_ts {
    ($self:ident, $($param:ident)*) => {$(
        let $param = $self.$param.as_ref().map(|l| quote!($param: #l,));
    )*};
}

impl ToTokens for Format {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root();

        map_to_ts!(self, prefix left_sur right_sur chip show_default suffix line_brk);

        let opt_default = if self.some_omitted {
            Some(quote!(..Default::default()))
        } else {
            None
        };

        tokens.extend(quote!(#root::field::Format {
            #prefix
            #left_sur
            #right_sur
            #chip
            #show_default
            #suffix
            #line_brk
            #opt_default
        }));
    }
}

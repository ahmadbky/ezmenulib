//! Module used to easily define macro attributes, and the keywords used by them.
//!
//! Macro attributes, as meta-item, become pretty closer to each other, because they consist of
//! many parameters defined in any order by the user. To avoid repetition in the macros definition
//! code, we use the [`define_attr`] macro, to define a struct that will represent the attribute
//! of an item, with its fields used as the attribute parameters.
//!
//! This macro generates the attribute parsing code that checks if a parameter hasn't been provided
//! twice, and also if there isn't an other parameter that is in conflict with it, thank to the
//! `without field0; field1; ...` syntax. Parameters that enter in conflict are for example
//! `title = "..."` and `no_title`.

use proc_macro2::Span;
use proc_macro_error::abort;
use syn::{
    custom_keyword,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Index, LitBool, LitStr, Path, Token,
};

use concat_idents::concat_idents as id;

use crate::{
    format::Format,
    menu::{MapWith, MappedWith, RawBlockAttr},
    prompted::{promptable::RawSelectedField, FunctionExpr},
    utils::Case,
};

/// Util macro used to define the keywords used by the macros attributes.
macro_rules! define_keywords {
    {
        // param = value
        'eq: $( $eq:ident: $eq_ty:ty, )*
        // param(value)
        'par: $( $par:ident: $par_ty:ty, )*
        // param
        'unit: $( $unit:ident, )*
        // custom parse
        'else: $( $else:ident($input:ident) -> $else_ty:ty $block:block ),*
    } => {
        $(
            custom_keyword!($eq);
            id!(parse_eq = parse_, $eq {
                pub(crate) fn parse_eq(input: ParseStream) -> syn::Result<Option<$eq_ty>> {
                    input.parse::<$crate::kw::$eq>()?;
                    input.parse::<::syn::Token![=]>()?;
                    input.parse().map(Some)
                }
            });
            id!(duplicate_eq = duplicate_, $eq {
                pub(crate) fn duplicate_eq(v: &Option<$eq_ty>) -> bool {
                    v.is_some()
                }
            });
        )*

        $(
            custom_keyword!($par);
            id!(parse_par = parse_, $par {
                pub(crate) fn parse_par(input: ParseStream) -> syn::Result<Option<$par_ty>> {
                    input.parse::<$crate::kw::$par>()?;
                    let content;
                    syn::parenthesized!(content in input);
                    content.parse().map(Some)
                }
            });
            id!(duplicate_par = duplicate_, $par {
                pub(crate) fn duplicate_par(v: &Option<$par_ty>) -> bool {
                    v.is_some()
                }
            });
        )*

        $(
            custom_keyword!($unit);
            id!(parse_unit = parse_, $unit {
                pub(crate) fn parse_unit(input: ParseStream) -> syn::Result<bool> {
                    input.parse::<$crate::kw::$unit>()?;
                    Ok(true)
                }
            });
            id!(duplicate_unit = duplicate_, $unit {
                pub(crate) fn duplicate_unit(v: &bool) -> bool {
                    *v
                }
            });
        )*

        $(
            custom_keyword!($else);
            id!(parse_else = parse_, $else {
                pub(crate) fn parse_else($input: ParseStream) -> syn::Result<Option<$else_ty>> {
                    Ok(Some($block))
                }
            });
            id!(duplicate_else = duplicate_, $else {
                pub(crate) fn duplicate_else(v: &Option<$else_ty>) -> bool {
                    v.is_some()
                }
            });
        )*
    };
}

#[inline(never)]
fn _abort_tui_feature(sp: Span) -> ! {
    abort!(sp, "the `tui` feature must be enabled to use this keyword");
}

// We don't use the define_keywords macro to provide the `tui` unit parameter,
// because it depends of the `tui` feature.
custom_keyword!(tui);

pub(crate) fn parse_tui(input: ParseStream) -> syn::Result<bool> {
    let _sp = input.parse::<tui>()?.span;
    if cfg!(not(feature = "tui")) {
        _abort_tui_feature(_sp);
    }
    Ok(true)
}

pub(crate) fn duplicate_tui(v: &bool) -> bool {
    *v
}

define_keywords! {
    'eq: case: Case, title: LitStr, msg: LitStr, example: LitStr, sep: LitStr,
        prefix: LitStr, left_sur: LitStr, right_sur: LitStr, chip: LitStr,
        show_default: LitBool, suffix: LitStr, line_brk: LitBool, path: Path,
        styled_title: Expr, title_alignment: Expr, border_style: Expr, style: Expr,
        borders: Expr, border_type: Expr,
    'par: fmt: Format, until: FunctionExpr, or_val: LitStr, or_env: LitStr, map: FunctionExpr,
        mapped_with: MappedWith, map_with: MapWith,
    'unit: no_title, nodoc, raw, optional, or_default, flatten, basic_example, password,
        parent, quit, once,
    'else:
        block(input) -> RawBlockAttr {
            let _sp = input.parse::<block>()?.span;
            if cfg!(not(feature = "tui")) {
                _abort_tui_feature(_sp);
            }
            let content;
            syn::parenthesized!(content in input);
            content.parse()?
        },
        back(input) -> Index {
            let id = input.parse::<back>()?;
            if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                content.parse()?
            } else {
                Index { index: 1, span: id.span }
            }
        },
        or_env_with(input) -> (LitStr, LitStr) {
            input.parse::<or_env_with>()?;
            let content;
            syn::parenthesized!(content in input);
            let var = content.parse()?;
            content.parse::<Token![,]>()?;
            let sep = content.parse()?;
            (var, sep)
        },
        select(input) -> Punctuated<RawSelectedField, Token![,]> {
            input.parse::<select>()?;
            let content;
            syn::parenthesized!(content in input);
            content.parse_terminated(Parse::parse)?
        },
        default(input) -> proc_macro2::Span {
            input.parse::<default>()?.span
        },
        mapped(input) -> (Path, Punctuated<Expr, Token![,]>) {
            input.parse::<mapped>()?;
            let content;
            syn::parenthesized!(content in input);
            let id = content.parse()?;
            let args = if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
                content.parse_terminated(Parse::parse)?
            } else {
                Punctuated::new()
            };
            (id, args)
        }
}

/// Macro used to define a struct that will contain the data of an macro attribute.
/// 
/// See the [module](crate::kw) documentation for more details.
// FIXME: Maybe forbid the useless commas in `#[attr(,,,,,,, ...)]`?
macro_rules! define_attr {
    {
        $(#[$docs:meta])*
        $pub:vis $Attr:ident {$(
            $field:ident: $ty:ty $(; without $($cond:expr);*)?,
        )*}
    } => {
        $(#[$docs])*
        #[derive(Clone, Default, Debug)]
        $pub struct $Attr {$(
            $field: $ty,
        )*}

        impl ::syn::parse::Parse for $Attr {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                use ::concat_idents::concat_idents as id;
                let mut ret = Self::default();

                while !input.is_empty() {
                    let next = input.lookahead1();

                    match () {
                        $(_ if next.peek($crate::kw::$field) => match () {
                            _ if id!(duplicate_field = duplicate_, $field {
                                $crate::kw::duplicate_field(&ret.$field)
                            }) => {
                                $crate::utils::abort_duplicate_parameter(input.span(), stringify!($field));
                            }
                            $($(
                                _ if id!(duplicate_cond = duplicate_, $cond {
                                    $crate::kw::duplicate_cond(&ret.$cond)
                                }) => {
                                    $crate::utils::abort_conflict_param(input.span(), stringify!($field), stringify!($cond));
                                }
                            )*)?

                            _ => id!(parse_field = parse_, $field {
                                ret.$field = $crate::kw::parse_field(input)?
                            }),
                        })*
                        // The input input stream isn't empty because we entered the while loop,
                        // so the next tokentree must only be a comma.
                        _ if !input.peek(::syn::Token![,]) => return ::syn::Result::Err(next.error()),
                        _ => {
                            input.parse::<::core::option::Option<::syn::Token![,]>>()?;
                        }
                    }
                }

                Ok(ret)
            }
        }
    }
}

pub(crate) use define_attr;

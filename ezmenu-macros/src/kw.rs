use syn::{
    custom_keyword,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Index, LitBool, LitStr, Path, Token,
};

use concat_idents::concat_idents as id;

use crate::{
    format::Format,
    menu::{MapWith, MappedWith},
    prompted::{promptable::RawSelectedField, FunctionExpr},
    utils::Case,
};

macro_rules! define_keywords {
    {
        'eq: $( $eq:ident: $eq_ty:ty, )*
        'par: $( $par:ident: $par_ty:ty, )*
        'unit: $( $unit:ident, )*
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

define_keywords! {
    'eq: case: Case, title: LitStr, msg: LitStr, example: LitStr, sep: LitStr,
        prefix: LitStr, left_sur: LitStr, right_sur: LitStr, chip: LitStr,
        show_default: LitBool, suffix: LitStr, line_brk: LitBool,
    'par: fmt: Format, until: FunctionExpr, or_val: LitStr, or_env: LitStr, map: FunctionExpr,
        mapped_with: MappedWith, map_with: MapWith,
    'unit: no_title, nodoc, raw, optional, or_default, flatten, tui, basic_example, password,
        parent, quit, once,
    'else:
        back(input) -> Index {
            let id = input.parse::<back>()?;
            if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                content.parse()?
            } else {
                Index { index: 0, span: id.span }
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

macro_rules! define_attr {
    {
        $pub:vis $Attr:ident {$(
            $field:ident: $ty:ty $(; without $($cond:expr);*)?,
        )*}
    } => {
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

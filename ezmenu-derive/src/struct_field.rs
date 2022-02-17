use crate::*;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Field, Lit, LitBool, LitStr, Meta, NestedMeta, Path};

/// Wrapper used for the expansion of the `StructFieldFormatting` struct instantiation.
pub(crate) struct FieldFormatting {
    pub chip: Option<LitStr>,
    pub prefix: Option<LitStr>,
    pub new_line: Option<LitBool>,
    pub default: Option<LitBool>,
    // used to know if at least one parameter have been provided
    // so if we instantiate `StructFieldFormatting` or not
    pub custom_fmt: bool,
    // used to know if some parameters have been omitted in meta
    // so if we use `Default::default()` method or not
    pub some_omitted: bool,
}

impl Default for FieldFormatting {
    fn default() -> Self {
        Self {
            chip: None,
            prefix: None,
            new_line: None,
            default: None,
            // no parameter provided
            custom_fmt: false,
            // so all omitted
            some_omitted: true,
        }
    }
}

macro_rules! map_to_ts {
    ($self:expr, $($id:ident)*) => {
        $(let $id = $self.$id.as_ref().map(|lit| quote!($id: #lit,));)*
    }
}

impl ToTokens for FieldFormatting {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.custom_fmt {
            return;
        }

        map_to_ts!(self, chip prefix new_line default);

        let call_default = if self.some_omitted {
            quote!(..Default::default())
        } else {
            TokenStream::new()
        };

        tokens.extend(quote! {
            .fmt(::ezmenu::StructFieldFormatting {
                #chip
                #prefix
                #new_line
                #default
                #call_default
            })
        });
    }
}

/// Wrapper used for the expansion of the `StructField::default` method call.
/// It stringifies the literal value if not already a string literal.
struct DefaultValue(Lit);

impl ToTokens for DefaultValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let default = &self.0;
        tokens.extend(if let Lit::Str(_) = default {
            quote! {
                .default(#default)
            }
        } else {
            quote! {
                .default(stringify!(#default))
            }
        });
    }
}

/// Enum used to distinguish between simple output fields and mapped output fields.
pub(crate) enum FieldMenuInitKind {
    /// Simply ask the value then returns it.
    Simple(Ident),
    /// After the user returned the value, call the function to map the value.
    Mapped(Ident, Path),
}

impl ToTokens for FieldMenuInitKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::Simple(ident) => quote! {
                #ident: menu.next()?,
            },
            Self::Mapped(ident, func) => quote! {
                #ident: menu.next_map(#func)?,
            },
        })
    }
}

/// Struct defining the field behavior description.
#[derive(Default)]
struct MetaFieldDesc {
    msg: Option<LitStr>,
    default: Option<Lit>,
    then: Option<Path>,
    fmt: FieldFormatting,
}

/// Binds the argument of a list meta attribute to one
/// of the fields of the output MetaFieldDesc.
///
/// It is not the same as a name-value attribute because
/// the name-value accepts only literal values, so cannot build
/// the `then` argument for example. Moreover, the argument is
/// represented by a NestedMeta.
fn parse_arg_nested(
    MetaFieldDesc {
        ref mut msg,
        ref mut default,
        ref mut then,
        fmt:
            FieldFormatting {
                ref mut chip,
                ref mut prefix,
                ref mut new_line,
                default: ref mut disp_default,
                ..
            },
    }: &mut MetaFieldDesc,
    arg: String,
    nested: &NestedMeta,
    span: NestedMeta,
) {
    match arg.as_str() {
        s @ "msg" => run_nested_str(s, nested, msg),
        s @ "default" => run_nested(s, nested, default),
        s @ "then" => run_nested_path(s, nested, then),
        s @ "chip" => run_nested_str(s, nested, chip),
        s @ "prefix" => run_nested_str(s, nested, prefix),
        s @ "new_line" => run_nested_bool(s, nested, new_line),
        s @ "display_default" => run_nested_bool(s, nested, disp_default),
        s => abort_invalid_arg_name(span, s),
    }
}

/// Binds the argument of a name-value meta attribute to one
/// of the fields of the output MetaFieldDesc.
///
/// It is not the same as a list meta attribute, because
/// it only accepts literal values.
fn parse_arg_nv(
    MetaFieldDesc {
        ref mut msg,
        ref mut default,
        fmt:
            FieldFormatting {
                ref mut chip,
                ref mut prefix,
                ref mut new_line,
                default: ref mut disp_default,
                ..
            },
        ..
    }: &mut MetaFieldDesc,
    arg: String,
    lit: Lit,
    span: NestedMeta,
) {
    match arg.as_str() {
        s @ "msg" => run_nv_str(s, lit, msg),
        "default" => *default = Some(lit),
        s @ "chip" => run_nv_str(s, lit, chip),
        s @ "prefix" => run_nv_str(s, lit, prefix),
        s @ "new_line" => run_nv_bool(s, lit, new_line),
        s @ "display_default" => run_nv_bool(s, lit, disp_default),
        s => abort_invalid_arg_name(span, s),
    }
}

/// Implementation used to parse the inner parameters
/// of a `menu` attribute
/// into the description of the menu.
impl From<Meta> for MetaFieldDesc {
    fn from(meta: Meta) -> Self {
        let mut desc = MetaFieldDesc::default();

        parse(&mut desc, parse_arg_nested, parse_arg_nv, meta);

        let MetaFieldDesc {
            msg,
            default,
            then,
            fmt:
                FieldFormatting {
                    chip,
                    prefix,
                    new_line,
                    default: disp_default,
                    ..
                },
        } = desc;

        // we need to declare variables here so fmt params are not moved
        let custom_fmt =
            chip.is_some() || prefix.is_some() || new_line.is_some() || disp_default.is_some();
        let some_omitted =
            !(chip.is_some() && prefix.is_some() && new_line.is_some() && disp_default.is_some());

        Self {
            msg,
            default,
            then,
            fmt: FieldFormatting {
                chip,
                prefix,
                new_line,
                default: disp_default,
                custom_fmt,
                some_omitted,
            },
        }
    }
}

/// Wrapper used for the expansion of the `StructField` struct methods calls.
pub(crate) struct FieldMenuInit {
    // if no lit provided, using field ident
    msg: LitStr,
    default: Option<DefaultValue>,
    fmt: FieldFormatting,
    // appended to tokenstream at the end
    // when calling `Menu::next` method
    pub kind: FieldMenuInitKind,
}

impl From<Field> for FieldMenuInit {
    fn from(field: Field) -> Self {
        // field is supposed to be named
        let ident = field.ident.unwrap();

        let desc = match get_meta_attr(field.attrs) {
            Some(meta) => MetaFieldDesc::from(meta),
            _ => Default::default(),
        };

        let msg = desc
            .msg
            .unwrap_or_else(|| LitStr::new(format!("{}", ident).as_str(), ident.span()));
        let default = desc.default.map(DefaultValue);
        let kind = match desc.then {
            Some(func) => FieldMenuInitKind::Mapped(ident, func),
            _ => FieldMenuInitKind::Simple(ident),
        };

        Self {
            msg,
            default,
            fmt: desc.fmt,
            kind,
        }
    }
}

impl ToTokens for FieldMenuInit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        {
            let msg = &self.msg;
            tokens.extend(quote! {
                ::ezmenu::StructField::from(#msg)
            });
        }
        self.default.to_tokens(tokens);
        self.fmt.to_tokens(tokens);
    }
}

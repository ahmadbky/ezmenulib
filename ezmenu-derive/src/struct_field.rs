use crate::{abort_invalid_arg_name, abort_invalid_type, get_meta_attr, path_to_string, run};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{Field, Lit, LitBool, LitStr, Meta, MetaList, MetaNameValue, NestedMeta, Path};

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
    ($self:expr, $id:ident) => {
        let $id = $self.$id.as_ref().map(|lit| quote!($id: #lit,));
    }
}

impl ToTokens for FieldFormatting {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.custom_fmt {
            return;
        }

        map_to_ts!(self, chip);
        map_to_ts!(self, prefix);
        map_to_ts!(self, new_line);
        map_to_ts!(self, default);

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
        match self {
            Self::Simple(ident) => tokens.extend(quote! {
                #ident: menu.next()?,
            }),
            Self::Mapped(ident, func) => tokens.extend(quote! {
                #ident: menu.next_map(#func)?,
            }),
        }
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

/// Implementation used to parse the inner parameters
/// of a `menu` attribute
/// into the description of the menu
// FIXME: disable duplication of meta parsing for field and struct attributes
impl From<Meta> for MetaFieldDesc {
    fn from(meta: Meta) -> Self {
        // values edited at each iteration
        // (if the user provided them multiple times)
        let MetaFieldDesc {
            mut msg,
            mut default,
            mut then,
            fmt:
                FieldFormatting {
                    mut chip,
                    mut prefix,
                    mut new_line,
                    default: mut disp_default,
                    ..
                },
        } = Default::default();

        // root meta must be a list
        if let Meta::List(MetaList { nested, .. }) = meta {
            for nm in nested {
                match nm {
                    // in inner metas, if the meta type is a list,
                    // then it should contain only 1 nested meta as value
                    // like a path to a function, or a string literal for a message
                    NestedMeta::Meta(Meta::List(MetaList { path, nested, .. })) => {
                        // get the first nested meta inside parenthesis
                        let nested = nested.first();
                        let nested = match nested {
                            Some(nm) => nm,
                            _ => abort!(path, "value definition missing"),
                        };

                        match path_to_string(&path).as_str() {
                            s @ "msg" => run!(nested: msg, Str, nested, s),
                            s @ "default" => {
                                if let NestedMeta::Lit(lit) = nested {
                                    default = Some(lit.clone());
                                } else {
                                    abort_invalid_type(nested, s);
                                }
                            }
                            s @ "then" => {
                                if let NestedMeta::Meta(Meta::Path(path)) = nested {
                                    then = Some(path.clone());
                                } else {
                                    abort_invalid_type(nested, s);
                                }
                            }
                            s @ "chip" => run!(nested: chip, Str, nested, s),
                            s @ "prefix" => run!(nested: prefix, Str, nested, s),
                            s @ "new_line" => run!(nested: new_line, Bool, nested, s),
                            s @ "display_default" => run!(nested: disp_default, Bool, nested, s),
                            s => abort_invalid_arg_name(path, s),
                        }
                    }
                    // deconstructing to a path and a literal
                    NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) => {
                        match path_to_string(&path).as_str() {
                            s @ "msg" => run!(msg, Str, lit, s),
                            "default" => default = Some(lit.clone()),
                            s @ "chip" => run!(chip, Str, lit, s),
                            s @ "prefix" => run!(prefix, Str, lit, s),
                            s @ "new_line" => run!(new_line, Bool, lit, s),
                            s @ "display_default" => run!(disp_default, Bool, lit, s),
                            s => abort_invalid_arg_name(path, s),
                        }
                    }
                    _ => abort!(nm, "expected value definition"),
                }
            }
        } else {
            abort!(meta, "incorrect definition of menu attribute");
        }

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
            .unwrap_or(LitStr::new(format!("{}", ident).as_str(), ident.span()));
        let default = desc.default.map(|lit| DefaultValue(lit));
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

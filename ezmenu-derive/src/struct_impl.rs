use crate::*;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Lit, LitStr, Meta, NestedMeta};

/// Wrapper used for the expansion of the `StructMenu::title` method call.
struct MenuTitle(LitStr);

impl ToTokens for MenuTitle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let lit = &self.0;
        tokens.extend(quote! {
            .title(#lit)
        });
    }
}

/// Struct defining the main description of the menu behavior.
#[derive(Default)]
struct MetaMenuDesc {
    title: Option<LitStr>,
    fmt: FieldFormatting,
}

fn parse_arg_nested(
    MetaMenuDesc {
        ref mut title,
        fmt:
            FieldFormatting {
                ref mut chip,
                ref mut prefix,
                ref mut new_line,
                default: ref mut disp_default,
                ..
            },
    }: &mut MetaMenuDesc,
    arg: String,
    nested: &NestedMeta,
) {
    match arg.as_str() {
        s @ "title" => run_nested_str(s, nested, title),
        s @ "chip" => run_nested_str(s, nested, chip),
        s @ "prefix" => run_nested_str(s, nested, prefix),
        s @ "new_line" => run_nested_bool(s, nested, new_line),
        s @ "display_default" => run_nested_bool(s, nested, disp_default),
        s => abort_invalid_arg_name(nested, s),
    }
}

fn parse_arg_nv(
    MetaMenuDesc {
        ref mut title,
        fmt:
            FieldFormatting {
                ref mut chip,
                ref mut prefix,
                ref mut new_line,
                default: ref mut disp_default,
                ..
            },
    }: &mut MetaMenuDesc,
    arg: String,
    lit: Lit,
) {
    match arg.as_str() {
        s @ "title" => run_nv_str(s, lit, title),
        s @ "chip" => run_nv_str(s, lit, chip),
        s @ "prefix" => run_nv_str(s, lit, prefix),
        s @ "new_line" => run_nv_bool(s, lit, new_line),
        s @ "display_default" => run_nv_bool(s, lit, disp_default),
        s => abort_invalid_arg_name(s, s),
    }
}

/// Implementation used to parse the inner parameters
/// of a `menu` attribute
/// into the description of the menu
impl From<Meta> for MetaMenuDesc {
    fn from(meta: Meta) -> Self {
        let mut desc = MetaMenuDesc::default();

        parse(&mut desc, parse_arg_nested, parse_arg_nv, meta);

        let MetaMenuDesc {
            title,
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
            title,
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

/// Wrapper used for the expansion of the `StructMenu::title`
/// and `StructMenu::fmt` method calls
struct AllMenuInit {
    title: Option<MenuTitle>,
    fmt: FieldFormatting,
}

impl ToTokens for AllMenuInit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.title.to_tokens(tokens);
        self.fmt.to_tokens(tokens);
    }
}

impl From<Meta> for AllMenuInit {
    fn from(meta: Meta) -> Self {
        // wrapping to "tokenable" values
        let desc = MetaMenuDesc::from(meta);
        let title = desc.title.map(MenuTitle);

        Self {
            title,
            fmt: desc.fmt,
        }
    }
}

/// The whole menu instantiation expansion.
/// It constructs the menu and the fields it contains.
pub(crate) struct MenuInit {
    all: Option<AllMenuInit>,
    pub fields: Vec<FieldMenuInit>,
}

impl ToTokens for MenuInit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.all.to_tokens(tokens);
        let fields = &self.fields;
        tokens.extend(quote! {#(
            .with_field(#fields)
        )*})
    }
}

impl MenuInit {
    pub fn new(struct_attr: Option<Meta>, fields: Vec<FieldMenuInit>) -> Self {
        Self {
            all: struct_attr.map(AllMenuInit::from),
            fields,
        }
    }
}

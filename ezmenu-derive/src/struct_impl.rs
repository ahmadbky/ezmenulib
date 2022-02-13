use crate::{
    abort_invalid_arg_name, abort_invalid_type, path_to_string, run, FieldFormatting, FieldMenuInit,
};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{Lit, LitStr, Meta, MetaList, MetaNameValue, NestedMeta};

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

/// Implementation used to parse the inner parameters
/// of a `menu` attribute
/// into the description of the menu
// FIXME: disable duplication of meta parsing for field and struct attributes
impl From<Meta> for MetaMenuDesc {
    fn from(meta: Meta) -> Self {
        // values edited at each iteration
        // (if the user provided them multiple times)
        let MetaMenuDesc {
            mut title,
            fmt:
                FieldFormatting {
                    mut chip,
                    mut prefix,
                    mut new_line,
                    mut default,
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
                            s @ "title" => run!(nested: title, Str, nested, s),
                            s @ "chip" => run!(nested: chip, Str, nested, s),
                            s @ "prefix" => run!(nested: prefix, Str, nested, s),
                            s @ "new_line" => run!(nested: new_line, Bool, nested, s),
                            s @ "display_default" => run!(nested: default, Bool, nested, s),
                            s => abort_invalid_arg_name(path, s),
                        }
                    }
                    // deconstructing to a path and a literal
                    NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) => {
                        match path_to_string(&path).as_str() {
                            s @ "title" => run!(title, Str, lit, s),
                            s @ "chip" => run!(chip, Str, lit, s),
                            s @ "prefix" => run!(prefix, Str, lit, s),
                            s @ "new_line" => run!(new_line, Bool, lit, s),
                            s @ "display_default" => run!(default, Bool, lit, s),
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
            chip.is_some() || prefix.is_some() || new_line.is_some() || default.is_some();
        let some_omitted =
            !(chip.is_some() && prefix.is_some() && new_line.is_some() && default.is_some());

        Self {
            title,
            fmt: FieldFormatting {
                chip,
                prefix,
                new_line,
                default,
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
        let title = desc.title.map(|lit| MenuTitle(lit));

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
            all: struct_attr.map(|attr| AllMenuInit::from(attr)),
            fields,
        }
    }
}

//! Module that manages the expansion of the Prompted derive macro for an enum.

use proc_macro2::{Delimiter, Group, Punct, Spacing, Span, TokenStream};
use proc_macro_error::{abort, set_dummy};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Expr, Field, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, Index, LitStr,
    Token, Variant,
};

use crate::{
    format::Format,
    kw::define_attr,
    utils::{
        get_attr_with_args, get_first_doc, get_lib_root, method_call, split_ident_camel_case,
        take_val, wrap_in_const, Case, MethodCall,
    },
};

define_attr! {
    /// Represents the prompted attribute of a unit variant.
    ///
    /// We save the default parameter keyword span to check at the end
    /// if there are many default values, to span the error.
    UnitAttr {
        msg: Option<LitStr>,
        default: Option<Span>,
        case: Option<Case>,
        nodoc: bool; without msg,
        raw: bool; without msg,
    }
}

/// Represents a single binding for a variant that has fields.
///
/// The identifier is optional, depending on the named fields of the variant.
#[derive(Clone)]
struct Binding {
    id: Option<Ident>,
    val: Expr,
}

impl Binding {
    fn parse_(input: ParseStream, named: bool) -> syn::Result<Self> {
        let id = if named {
            // We are expecting identifier that is not a keyword.
            if input.peek(Ident) {
                let id = input.parse()?;
                input.parse::<Token![:]>()?;
                Some(id)
            } else {
                abort!(input.span(), "expected an identifier");
            }
        } else {
            None
        };

        let val = input.parse()?;

        Ok(Self { id, val })
    }

    // We use distinct functions because `ParseBuffer::parse_terminated` only takes `fn`s
    // so we can't capture variables with closures :|

    fn parse_unnamed(input: ParseStream) -> syn::Result<Self> {
        Self::parse_(input, false)
    }

    fn parse_named(input: ParseStream) -> syn::Result<Self> {
        Self::parse_(input, true)
    }
}

impl ToTokens for Binding {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(id) = &self.id {
            id.to_tokens(tokens);
            tokens.append(Punct::new(':', Spacing::Alone));
        }

        self.val.to_tokens(tokens);
    }
}

/// Represents a single `("msg", bounds...)` in the attribute of a variant that has fields.
///
/// It contains the span of the `default` identifier if provided in `default(...)`.
struct SelectedField {
    lit: LitStr,
    bounds: Punctuated<Binding, Token![,]>,
    default: Option<Span>,
}

impl SelectedField {
    fn parse(input: ParseStream, named: bool) -> syn::Result<Self> {
        let default = if input.peek(Ident::peek_any) {
            // Here, we expect the `default` identifier which is a keyword.
            let id = input.parse::<Ident>()?;
            if id != "default" {
                abort!(id, "unexpected identifier")
            }
            Some(id.span())
        } else {
            None
        };

        // The content inside the parenthesis.
        let content;
        parenthesized!(content in input);
        let lit = content.parse()?;
        content.parse::<Token![,]>()?;
        let bounds = content.parse_terminated(if named {
            Binding::parse_named
        } else {
            Binding::parse_unnamed
        })?;

        Ok(Self {
            lit,
            bounds,
            default,
        })
    }
}

/// Represents the select attribute of a variant that has named fields.
struct NamedAttr {
    fields: Punctuated<SelectedField, Token![,]>,
}

impl Parse for NamedAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fields = input.parse_terminated(|i| SelectedField::parse(i, true))?;
        Ok(Self { fields })
    }
}

/// Represents the select attribute of a variant that has unnamed fields.
struct UnnamedAttr {
    fields: Punctuated<SelectedField, Token![,]>,
}

impl Parse for UnnamedAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fields = input.parse_terminated(|i| SelectedField::parse(i, false))?;
        Ok(Self { fields })
    }
}

/// Represents the entry of a selected field
/// when expanding inside the `Selectable::values` function.
///
/// An entry is basically a tuple of a string literal and a bound value to it.
#[derive(Clone)]
struct Entry {
    /// The identifier of the pointed variant.
    id: Ident,
    /// The literal of the entry.
    lit: String,
    /// Used to choose between parenthesized or braced group when converting to tokens
    /// to put the bound values inside.
    named: bool,
    /// The bound values to the variant if it has fields.
    bounds: Punctuated<Binding, Token![,]>,
    /// Contains the span of the `default` ident if provided, for error handling.
    default: Option<Span>,
}

impl Entry {
    /// Creates an entry from the `SelectedField`, the identifier of the pointed variant,
    /// and if the variant has named variants or not.
    fn new(f: SelectedField, id: Ident, named: bool) -> Self {
        Self {
            id,
            lit: f.lit.value(),
            named,
            bounds: f.bounds,
            default: f.default,
        }
    }

    fn from_var(id: Ident, lit: String, default: Option<Span>) -> Self {
        Self {
            id,
            lit,
            named: false,
            bounds: Punctuated::new(),
            default,
        }
    }
}

impl ToTokens for Entry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let lit = &self.lit;
        let id = &self.id;
        let mut out = quote! {
            #lit, Self::#id
        };

        if !self.bounds.is_empty() {
            let delim = if self.named {
                Delimiter::Brace
            } else {
                Delimiter::Parenthesis
            };

            let g = Group::new(delim, self.bounds.to_token_stream());
            out.append(g);
        }

        let g = Group::new(Delimiter::Parenthesis, out);
        tokens.append(g);
    }
}

/// Util function used to abort on variants that have fields but without a `select` attribute.
fn abort_unbounds_fields(named: bool, fields: &Punctuated<Field, Token![,]>) -> ! {
    let values_sample = if fields.len() == 1 {
        let field = &fields[0];
        if named {
            format!("{}: value", field.ident.as_ref().unwrap())
        } else {
            "value".to_owned()
        }
    } else if named {
        fields
            .iter()
            .enumerate()
            .map(|(i, v)| format!("{}: value{i}", v.ident.as_ref().unwrap()))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        (0..fields.len())
            .map(|i| format!("value{i}"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    abort!(
        fields,
        "expected variant to have bound values in select attribute";
        note = "this variant has fields, thus needs a `#[select(...)]` attribute \
        to map at least one value to it";
        help = "you might want to add an attribute to this variant \
        with `#[select((\"field message\", {}), ...)]`", values_sample
    );
}

/// Represents the `select` metadata of the `Select`-derived enum variant.
///
/// It is built up from a variant, to check its optional fields, and its optional attribute.
struct Select {
    entries: Vec<Entry>,
}

impl Select {
    fn new(fields: Punctuated<SelectedField, Token![,]>, id: Ident, named: bool) -> Self {
        let entries = fields
            .into_iter()
            .map(|f| Entry::new(f, id.clone(), named))
            .collect();

        Self { entries }
    }

    fn from_variant_with(var: Variant, global_case: Option<Case>) -> Self {
        // We build the entries according to the fields type of the variant.
        match &var.fields {
            // If it is a unit variant, we return the previous vector of entries with only one entry.
            Fields::Unit => {
                let (lit, default) = match get_attr_with_args(&var.attrs, "prompt").map(take_val) {
                    Some(UnitAttr {
                        msg,
                        default,
                        case,
                        nodoc,
                        raw,
                    }) => {
                        let lit = msg
                            .map(|l| l.value())
                            .or_else(|| {
                                if nodoc {
                                    None
                                } else {
                                    get_first_doc(&var.attrs)
                                }
                            })
                            .unwrap_or_else(|| {
                                if raw {
                                    var.ident.to_string()
                                } else {
                                    split_ident_camel_case(&var.ident)
                                }
                            });
                        (case.or(global_case).unwrap_or_default().map(lit), default)
                    }
                    None => (
                        global_case.unwrap_or_default().map(
                            get_first_doc(&var.attrs)
                                .unwrap_or_else(|| split_ident_camel_case(&var.ident)),
                        ),
                        None,
                    ),
                };
                let entries = vec![Entry::from_var(var.ident, lit, default)];
                Self { entries }
            }
            // Otherwise, we create a new vector with the entries from the fields of the variant.
            Fields::Named(FieldsNamed { named, .. }) => {
                match get_attr_with_args(&var.attrs, "prompt").map(take_val) {
                    Some(NamedAttr { fields }) => Self::new(fields, var.ident, true),
                    None => abort_unbounds_fields(true, named),
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                match get_attr_with_args(&var.attrs, "prompt").map(take_val) {
                    Some(UnnamedAttr { fields }) => Self::new(fields, var.ident, false),
                    None => abort_unbounds_fields(false, unnamed),
                }
            }
        }
    }
}

define_attr! {
    /// Represents the prompted attribute of the enum.
    RootAttr {
        msg: Option<LitStr>,
        fmt: Option<Format>,
        case: Option<Case>,
        nodoc: bool; without msg,
        raw: bool; without msg,
    }
}

/// Represents the global data of the enum, meaning the message displayed,
/// which can is its identifier name by default, and its optional specific format.
struct RootData {
    case: Option<Case>,
    msg: String,
    fmt: Option<Format>,
}

impl RootData {
    fn new(name: &Ident, attrs: &[Attribute]) -> Self {
        let (case, msg, fmt) = match get_attr_with_args(attrs, "prompt").map(take_val) {
            Some(RootAttr {
                msg,
                fmt,
                nodoc,
                case,
                raw,
            }) => {
                let msg = msg
                    .map(|l| l.value())
                    .or_else(|| if nodoc { None } else { get_first_doc(attrs) })
                    .unwrap_or_else(|| {
                        if raw {
                            name.to_string()
                        } else {
                            split_ident_camel_case(name)
                        }
                    });
                (case, case.unwrap_or_default().map(msg), fmt)
            }
            None => (
                None,
                get_first_doc(attrs).unwrap_or_else(|| split_ident_camel_case(name)),
                None,
            ),
        };

        Self { case, msg, fmt }
    }
}

/// Returns the optional token stream of the `Selectable::default` function call.
///
/// It browses the whole iterator to check that there is at most one default value provided,
/// otherwise it aborts the macro expansion.
fn get_default_fn<I: Iterator<Item = Entry>>(input: I) -> Option<MethodCall<Index>> {
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

/// Expands the `derive(Prompted)` macro for an enum.
///
/// The expansion consists of the implementation of the `Selectable` trait for the given enum,
/// with the given variants, and the `Prompted` trait.
pub(crate) fn build_select(
    used: TokenStream,
    attrs: Vec<Attribute>,
    name: Ident,
    gens: Generics,
    variants: Punctuated<Variant, Token![,]>,
) -> TokenStream {
    if !gens.params.is_empty() {
        abort!(gens, "derive(Prompted) only supports non-generic enums.");
    }

    // The name of the library.
    let root = get_lib_root().1;

    set_dummy(wrap_in_const(quote! {
        #[automatically_derived]
        impl #root::menu::Prompted for #name {
            fn from_values<__H: #root::menu::Handle>(_: &mut #root::menu::Values<__H>) -> #root::MenuResult<Self> {
                unimplemented!()
            }
        }

        #[automatically_derived]
        impl #root::field::Selectable<0> for #name {
            fn select() -> #root::field::Selected<'static, Self, 0> {
                #used
                unimplemented!()
            }
        }
    }));

    let data = RootData::new(&name, &attrs);

    // The message and optional `Selected::format` method call, retrieved from
    // the optional attribute and the name of the enum.
    let (msg, fmt_fn) = (data.msg, data.fmt.map(|f| method_call("format", f)));

    // We map the variants into an iterator of selectable entries.
    // A variant can have multiple entries if it has fields.
    let entries = variants
        .into_iter()
        .flat_map(|v| Select::from_variant_with(v, data.case).entries.into_iter());

    // We count the amount of entries to specify in the const generic argument
    // of the `Selectable` trait.
    let n = Index::from(entries.clone().count());
    // We retrieve the `Selectable::default` function expansion from the entries.
    let default_fn = get_default_fn(entries.clone());

    wrap_in_const(quote! {
        #[automatically_derived]
        impl #root::menu::Prompted for #name {
            fn from_values<__H: #root::menu::Handle>(__vals: &mut #root::menu::Values<__H>) -> #root::MenuResult<Self> {
                __vals.next(#root::field::Selectable::select())
            }
        }

        #[automatically_derived]
        impl #root::field::Selectable<#n> for #name {
            fn select() -> #root::field::Selected<'static, Self, #n> {
                #used
                #root::field::Selected::new(#msg, [#(#entries),*])
                #fmt_fn
                #default_fn
            }
        }
    })
}

use proc_macro2::{Delimiter, Group, Punct, Spacing, Span, TokenStream};
use proc_macro_error::{abort, abort_call_site};
use quote::{quote, ToTokens, TokenStreamExt};
use regex::Regex;
use syn::{
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Data, DataEnum, DeriveInput, Expr, Field, Fields, FieldsNamed, FieldsUnnamed, Ident,
    Index, Lit, LitStr, Meta, MetaNameValue, Token, Variant,
};

use crate::utils::{get_attr, get_attr_with_args, get_lib_root};

fn abort_invalid_ident<const N: usize>(id: Ident, valids: [&str; N]) -> ! {
    abort!(id, "expected one of: {}, got: `{}`", valids.join(", "), id);
}

/// Represents the kind of an identifier for an unit variant.
enum UnitKind {
    /// The `default` identifier.
    Default,
    /// The `msg` optional identifier with the provided string literal.
    Lit(LitStr),
}

/// Represents an identifier with its span for error handling for an unit variant.
struct Unit {
    span: Span,
    kind: UnitKind,
}

impl Parse for Unit {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident::peek_any) {
            // The provided identifier must be either `default` alone
            // or `msg` in `msg = "..."`.
            let id = input.parse::<Ident>()?;
            if id == "default" {
                Ok(Self {
                    span: id.span(),
                    kind: UnitKind::Default,
                })
            } else if id == "msg" {
                input.parse::<Token![=]>()?;
                let msg = input.parse::<LitStr>()?;
                Ok(Self {
                    span: msg.span(),
                    kind: UnitKind::Lit(msg),
                })
            } else {
                abort_invalid_ident(id, ["default", "msg"]);
            }
        } else {
            // Else, the next token must be a string literal to represent the given message.
            let msg = input.parse::<LitStr>()?;
            Ok(Self {
                span: msg.span(),
                kind: UnitKind::Lit(msg),
            })
        }
    }
}

fn abort_already_defined(span: Span) -> ! {
    abort!(span, "attribute already defined");
}

/// Represents the attribute of an unit variant, with its optional string literal and the
/// span of the `default` identifier if provided for error handling.
struct UnitAttr {
    lit: Option<LitStr>,
    default: Option<Span>,
}

impl Parse for UnitAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Here, we iterate over `Unit`s thrice
        // to check that there is maximum 2 distinct values provided.
        let mut lit = None;
        let mut default = None;

        let mut vals = Punctuated::<_, Token![,]>::parse_terminated(input)?.into_iter();

        // First iteration, we initialize the given value.
        match vals.next() {
            Some(Unit {
                kind: UnitKind::Default,
                span,
                ..
            }) => default = Some(span),
            Some(Unit {
                kind: UnitKind::Lit(l),
                ..
            }) => lit = Some(l),
            None => (),
        }

        // Second iteration, we check if the provided value hasn't already been initialized.
        match vals.next() {
            Some(Unit {
                kind: UnitKind::Default,
                span,
                ..
            }) if default.is_none() => default = Some(span),
            Some(Unit {
                kind: UnitKind::Lit(l),
                ..
            }) if lit.is_none() => lit = Some(l),
            Some(u) => abort_already_defined(u.span),
            None => (),
        }

        // Third iteration, we abort because there is maximum 2 values that can be provided.
        if let Some(u) = vals.next() {
            abort_already_defined(u.span);
        }

        Ok(Self { lit, default })
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
            if id == "default" {
                Some(id.span())
            } else {
                abort!(id, "unexpected identifier")
            }
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

impl From<Ident> for Entry {
    /// We use this implementation to get an entry only from an unit variant.
    ///
    /// It uses the identifier of the variant as string literal, by splitting the words.
    fn from(id: Ident) -> Self {
        let lit = Regex::new("([a-z])([A-Z])")
            .unwrap_or_else(|e| {
                abort!(
                    id,
                    "unexpected error while splitting ident words with space as delimiter: {}",
                    e
                )
            })
            .replace_all(id.to_string().as_str(), "$1 $2")
            .into_owned();

        Self {
            id,
            lit,
            named: false,
            bounds: Punctuated::new(),
            default: None,
        }
    }
}

/// Util function used to abort on variants that have fields but without a `select` attribute.
fn abort_unbounds_fields(fields: &Punctuated<Field, Token![,]>) -> ! {
    abort!(
        fields,
        "expected variant to have bound values in select attribute";
        note = "this variant has fields, thus needs a `#[select(...)]` attribute \
        to map at least one value to it";
        help = "you might want to add an attribute to this variant with `#[select((\"...\", ...), ...)]`"
    )
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
}

impl From<Variant> for Select {
    fn from(var: Variant) -> Self {
        let mut entries = vec![Entry::from(var.ident.clone())];

        // We check the first doc comment for field name.
        match get_attr(&var.attrs, "doc").and_then(|attr| attr.parse_meta().ok()) {
            Some(Meta::NameValue(MetaNameValue {
                lit: Lit::Str(lit), ..
            })) if matches!(var.fields, Fields::Unit) => {
                entries[0].lit = lit.value().trim_start_matches(' ').to_owned();
            }
            _ => (),
        }

        // We build the entries according to the fields type of the variant.
        match &var.fields {
            // If it is a unit variant, we return the previous vector of entries with only one entry.
            Fields::Unit => {
                match get_attr_with_args(&var.attrs, "select") {
                    Some(UnitAttr { lit, default }) => {
                        match lit {
                            Some(lit) => entries[0].lit = lit.value(),
                            None => (),
                        }
                        entries[0].default = default;
                    }
                    None => (),
                }
                Self { entries }
            }
            // Otherwise, we create a new vector with the entries from the fields of the variant.
            Fields::Named(FieldsNamed { named, .. }) => {
                match get_attr_with_args(&var.attrs, "select") {
                    Some(NamedAttr { fields }) => Self::new(fields, var.ident, true),
                    None => abort_unbounds_fields(named),
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                match get_attr_with_args(&var.attrs, "select") {
                    Some(UnnamedAttr { fields }) => Self::new(fields, var.ident, false),
                    None => abort_unbounds_fields(unnamed),
                }
            }
        }
    }
}

/// Represents the kind of an identifier for the root attribute.
enum RootKind {
    /// The `msg` identifier, with the provided string literal.
    Msg(LitStr),
    /// The `fmt` identifier, with the provided `Format` instanciation.
    Fmt(Expr),
}

/// Represents an identifier with its span for error handling for the root attribute.
struct RootArg {
    span: Span,
    kind: RootKind,
}

impl Parse for RootArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) {
            // The provided identifier must be either `msg` or `fmt`.
            let id = input.parse::<Ident>()?;
            let span = id.span();
            input.parse::<Token![=]>()?;
            let kind = if id == "msg" {
                RootKind::Msg(input.parse()?)
            } else if id == "fmt" {
                RootKind::Fmt(input.parse()?)
            } else {
                abort_invalid_ident(id, ["msg", "fmt"]);
            };
            Ok(Self { kind, span })
        } else {
            // Else, the next token must be a string literal to represent the given message.
            let msg = input.parse::<LitStr>()?;
            let span = msg.span();
            Ok(Self {
                kind: RootKind::Msg(msg),
                span,
            })
        }
    }
}

/// Represents the attribute of the enum, with its optional string literal for the message
/// and its optional format specification.
struct RootAttr {
    msg: Option<LitStr>,
    fmt: Option<Expr>,
}

impl Parse for RootAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Here, we iterate over `Unit`s thrice
        // to check that there is maximum 2 distinct values provided.
        let mut msg = None;
        let mut fmt = None;

        let mut vals = Punctuated::<_, Token![,]>::parse_terminated(input)?.into_iter();

        // First iteration, we initialize the given value.
        match vals.next() {
            Some(RootArg {
                kind: RootKind::Fmt(f),
                ..
            }) => fmt = Some(f),
            Some(RootArg {
                kind: RootKind::Msg(m),
                ..
            }) => msg = Some(m),
            None => (),
        }

        // Second iteration, we check if the provided value hasn't already been initialized.
        match vals.next() {
            Some(RootArg {
                kind: RootKind::Fmt(f),
                ..
            }) if fmt.is_none() => fmt = Some(f),
            Some(RootArg {
                kind: RootKind::Msg(m),
                ..
            }) if msg.is_none() => msg = Some(m),
            Some(r) => abort_already_defined(r.span),
            None => (),
        }

        // Third iteration, we abort because there is maximum 2 values that can be provided.
        if let Some(r) = vals.next() {
            abort_already_defined(r.span);
        }

        Ok(Self { msg, fmt })
    }
}

/// Represents the global data of the enum, meaning the message displayed,
/// which can is its identifier name by default, and its optional specific format.
struct RootData {
    msg: String,
    fmt: Option<Expr>,
}

impl RootData {
    fn new(name: Ident, attrs: &[Attribute]) -> Self {
        match get_attr_with_args(attrs, "select") {
            Some(RootAttr { msg, fmt }) => Self {
                msg: msg.map(|l| l.value()).unwrap_or_else(|| name.to_string()),
                fmt,
            },
            None => Self {
                msg: name.to_string(),
                fmt: None,
            },
        }
    }
}

/// Returns the optional token stream of the `Selectable::default` function call.
///
/// It browses the whole iterator to check that there is at most one default value provided,
/// otherwise it aborts the macro expansion.
fn get_default_fn<I: Iterator<Item = Entry>>(input: I) -> Option<TokenStream> {
    let mut default = None;

    for (i, v) in input.enumerate() {
        if let Some(span) = v.default {
            if default.is_none() {
                default = Some(Index::from(i));
            } else {
                abort!(span, "there is already a default defined selected field");
            }
        }
    }

    default.map(|i| quote!(.default(#i)))
}

/// Returns the message of the `Selected` instanciation
/// and the optional token stream of the `Selected::format` method call.
fn get_data(name: Ident, attrs: &[Attribute]) -> (String, Option<TokenStream>) {
    let data = RootData::new(name, attrs);
    let fmt_fn = data.fmt.map(|e| quote!(.format(#e)));
    (data.msg, fmt_fn)
}

/// Expands the `derive(Select)` macro.
///
/// The expansion consists of the implementation of the `Selectable` trait for the given enum,
/// with the given variants.
pub fn build_select(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => abort_call_site!("derive(Select) only supports unit enums."),
    };

    // The message and optional `Selected::format` method call, retrieved from
    // the optional attribute and the name of the enum.
    let (msg, fmt_fn) = get_data(name.clone(), &input.attrs);

    // The name of the library.
    let root = get_lib_root();
    // We map the variants into an iterator of entries.
    // A variant can have multiple entries if it has fields.
    let entries = variants
        .into_iter()
        .map(|v| Select::from(v).entries.into_iter())
        .flatten();
    // We count the amount of entries to specify in the const generic argument
    // of the `Selectable` trait.
    let n = Index::from(entries.clone().count());
    // We retrieve the `Selectable::default` function expansion from the entries.
    let default_fn = get_default_fn(entries.clone());

    quote! {
        impl #root::field::Selectable<#n> for #name {
            fn select() -> #root::field::Selected<'static, Self, #n> {
                #root::field::Selected::new(#msg, [#(#entries),*])
                #fmt_fn
                #default_fn
            }
        }
    }
}

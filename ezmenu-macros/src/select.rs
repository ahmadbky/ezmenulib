use proc_macro2::{Delimiter, Group, Punct, Spacing, Span, TokenStream};
use proc_macro_error::{abort, abort_call_site};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Data, DataEnum, DeriveInput, Expr, Field, Fields, FieldsNamed, FieldsUnnamed, Ident,
    Index, LitStr, Token, Variant,
};

use crate::{
    format::Format,
    utils::{
        abort_invalid_ident, get_attr_with_args, get_first_doc, get_lib_root,
        split_ident_camel_case, to_str, Case,
    },
};

/// Represents the kind of an identifier for an unit variant.
enum UnitKind {
    /// The `default` identifier.
    Default,
    /// The `msg` optional identifier with the provided string literal.
    Msg(LitStr),
    /// The `case` optional identifier with the provided case specification.
    Case(Case),
    /// The `nodoc` identifier.
    NoDoc,
    /// The `raw` identifier.
    RawIdent,
}

/// Represents an identifier with its span for error handling for an unit variant.
struct UnitArg {
    span: Span,
    kind: UnitKind,
}

impl Parse for UnitArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident::peek_any) {
            let id = input.parse::<Ident>()?;
            let span = id.span();
            let kind = match to_str!(id) {
                "default" => UnitKind::Default,
                "msg" => {
                    input.parse::<Token![=]>()?;
                    UnitKind::Msg(input.parse()?)
                }
                "case" => {
                    input.parse::<Token![=]>()?;
                    UnitKind::Case(input.parse()?)
                }
                "nodoc" => UnitKind::NoDoc,
                "raw" => UnitKind::RawIdent,
                _ => abort_invalid_ident(id, &["default", "msg", "case", "nodoc", "raw"]),
            };

            Ok(Self { span, kind })
        } else {
            // Else, the next token must be a string literal to represent the given message.
            let msg = input.parse::<LitStr>()?;
            Ok(Self {
                span: msg.span(),
                kind: UnitKind::Msg(msg),
            })
        }
    }
}

/// Represents the attribute of an unit variant, with its optional string literal and the
/// span of the `default` identifier if provided for error handling.
// FIXME: Pretty same attribute as the root attribute
struct UnitAttr {
    lit: Option<LitStr>,
    default: Option<Span>,
    case: Option<Case>,
    nodoc: bool,
    raw_ident: bool,
}

impl Parse for UnitAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut lit = None;
        let mut default = None;
        let mut case = None;
        let mut nodoc = false;
        let mut raw_ident = false;

        let mut vals = Punctuated::<UnitArg, Token![,]>::parse_terminated(input)?.into_iter();
        let n = vals.len();

        for _ in 0..5.min(n) {
            match vals.next() {
                Some(arg) => match arg.kind {
                    UnitKind::Case(c) => case = Some(c),
                    UnitKind::Msg(m) => lit = Some(m),
                    UnitKind::NoDoc => nodoc = true,
                    UnitKind::Default => default = Some(arg.span),
                    UnitKind::RawIdent => raw_ident = true,
                },
                None => (),
            }
        }

        Ok(Self {
            lit,
            default,
            case,
            nodoc,
            raw_ident,
        })
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
                let (lit, default) = match get_attr_with_args(&var.attrs, "select") {
                    Some(UnitAttr {
                        lit,
                        default,
                        case,
                        nodoc,
                        raw_ident,
                    }) => {
                        let lit = lit
                            .map(|l| l.value())
                            .or_else(|| {
                                if nodoc {
                                    None
                                } else {
                                    get_first_doc(&var.attrs)
                                }
                            })
                            .unwrap_or_else(|| {
                                if raw_ident {
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
                match get_attr_with_args(&var.attrs, "select") {
                    Some(NamedAttr { fields }) => Self::new(fields, var.ident, true),
                    None => abort_unbounds_fields(true, named),
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                match get_attr_with_args(&var.attrs, "select") {
                    Some(UnnamedAttr { fields }) => Self::new(fields, var.ident, false),
                    None => abort_unbounds_fields(false, unnamed),
                }
            }
        }
    }
}

/// Represents a parameter in the root `select` attribute.
enum RootParam {
    /// The `msg` identifier, with the provided string literal.
    Msg(LitStr),
    /// The `fmt` identifier, with the provided `Format` instantiation.
    Fmt(Format),
    /// The `nodoc` identifier.
    NoDoc,
    /// The `case` identifier with the provided case specification.
    Case(Case),
    /// The `raw` identifier.
    RawIdent,
}

impl Parse for RootParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) {
            // The provided identifier must be either `msg` or `fmt`.
            let id = input.parse::<Ident>()?;

            Ok(match to_str!(id) {
                "msg" => {
                    input.parse::<Token![=]>()?;
                    Self::Msg(input.parse()?)
                }
                "fmt" => {
                    let content;
                    parenthesized!(content in input);
                    Self::Fmt(content.parse()?)
                }
                "nodoc" => Self::NoDoc,
                "case" => {
                    input.parse::<Token![=]>()?;
                    Self::Case(input.parse()?)
                }
                "raw" => Self::RawIdent,
                _ => abort_invalid_ident(id, &["msg", "fmt", "nodoc", "case"]),
            })
        } else {
            // Else, the next token must be a string literal to represent the given message.
            let msg = input.parse::<LitStr>()?;
            Ok(Self::Msg(msg))
        }
    }
}

/// Represents the `select` attribute of the enum, with its optional string literal for the message
/// and its optional format specification.
// FIXME: Pretty same attribute as the unit attribute
struct RootAttr {
    msg: Option<LitStr>,
    fmt: Option<Format>,
    case: Option<Case>,
    nodoc: bool,
    raw_ident: bool,
}

impl Parse for RootAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Here, we iterate over `Unit`s thrice
        // to check that there is maximum 2 distinct values provided.
        let mut msg = None;
        let mut fmt = None;
        let mut nodoc = false;
        let mut case = None;
        let mut raw_ident = false;

        let vals = Punctuated::<RootParam, Token![,]>::parse_terminated(input)?;
        let n = vals.len();
        let mut vals = vals.into_iter();

        for _ in 0..5.min(n) {
            match vals.next() {
                Some(arg) => match arg {
                    RootParam::Msg(m) => msg = Some(m),
                    RootParam::Fmt(f) => fmt = Some(f),
                    RootParam::NoDoc => nodoc = true,
                    RootParam::Case(c) => case = Some(c),
                    RootParam::RawIdent => raw_ident = true,
                },
                None => (),
            }
        }

        Ok(Self {
            msg,
            fmt,
            nodoc,
            case,
            raw_ident,
        })
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
    fn new(name: Ident, attrs: &[Attribute]) -> Self {
        let (case, msg, fmt) = match get_attr_with_args(attrs, "select") {
            Some(RootAttr {
                msg,
                fmt,
                nodoc,
                case,
                raw_ident,
            }) => {
                let msg = msg
                    .map(|l| l.value())
                    .or_else(|| if nodoc { None } else { get_first_doc(attrs) })
                    .unwrap_or_else(|| {
                        if raw_ident {
                            name.to_string()
                        } else {
                            split_ident_camel_case(&name)
                        }
                    });
                (case, case.unwrap_or_default().map(msg), fmt)
            }
            None => (
                None,
                get_first_doc(attrs).unwrap_or_else(|| split_ident_camel_case(&name)),
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

/// Expands the `derive(Select)` macro.
///
/// The expansion consists of the implementation of the `Selectable` trait for the given enum,
/// with the given variants.
pub fn build_select(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => abort_call_site!("derive(Select) only supports enums."),
    };

    let data = RootData::new(name.clone(), &input.attrs);

    // The message and optional `Selected::format` method call, retrieved from
    // the optional attribute and the name of the enum.
    let (msg, fmt_fn) = (data.msg, data.fmt.map(|e| quote!(.format(#e))));

    // The name of the library.
    let root = get_lib_root();
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

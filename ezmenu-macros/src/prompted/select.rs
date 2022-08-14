use proc_macro2::{Delimiter, Group, Punct, Spacing, Span, TokenStream};
use proc_macro_error::{abort, set_dummy};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
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
    utils::{
        abort_invalid_ident, define_attr, get_attr_with_args, get_first_doc, get_lib_root,
        method_call, split_ident_camel_case, take_val, to_str, Case, MethodCall, Sp,
    },
};

/// Represents a parameter in the prompt attribute of an unit variant.
enum UnitParam {
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

impl Parse for Sp<UnitParam> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Ident::peek_any) {
            let id = input.parse::<Ident>()?;
            let span = id.span();
            let val = match to_str!(id) {
                "default" => UnitParam::Default,
                "msg" => {
                    input.parse::<Token![=]>()?;
                    UnitParam::Msg(input.parse()?)
                }
                "case" => {
                    input.parse::<Token![=]>()?;
                    UnitParam::Case(input.parse()?)
                }
                "nodoc" => UnitParam::NoDoc,
                "raw" => UnitParam::RawIdent,
                _ => abort_invalid_ident(id, &["default", "msg", "case", "nodoc", "raw"]),
            };

            Self { span, val }
        } else {
            // Else, the next token must be a string literal to represent the given message.
            let msg = input.parse::<LitStr>()?;
            Self {
                span: msg.span(),
                val: UnitParam::Msg(msg),
            }
        })
    }
}

define_attr! {
    UnitParam(sp) -> UnitAttr {
        Msg(m) => lit: Option<LitStr> = None; if lit.is_none() => Some(m),
        Default => default: Option<Span> = None; if default.is_none() => Some(sp),
        Case(c) => case: Option<Case> = None; if case.is_none() => Some(c),
        NoDoc => nodoc: bool = false; if !nodoc && lit.is_none() => true,
        RawIdent => raw_ident: bool = false; if !raw_ident && lit.is_none() => true
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
    /// The identifier of the enum.
    enum_name: Ident,
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
    fn new(f: SelectedField, enum_name: Ident, id: Ident, named: bool) -> Self {
        Self {
            enum_name,
            id,
            lit: f.lit.value(),
            named,
            bounds: f.bounds,
            default: f.default,
        }
    }

    fn from_var(enum_name: Ident, id: Ident, lit: String, default: Option<Span>) -> Self {
        Self {
            enum_name,
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
        let name = &self.enum_name;
        let lit = &self.lit;
        let id = &self.id;
        let mut out = quote! {
            #lit, #name::#id
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
    fn new(
        enum_name: Ident,
        fields: Punctuated<SelectedField, Token![,]>,
        id: Ident,
        named: bool,
    ) -> Self {
        let entries = fields
            .into_iter()
            .map(|f| Entry::new(f, enum_name.clone(), id.clone(), named))
            .collect();

        Self { entries }
    }

    fn from_variant_with(var: Variant, global_case: Option<Case>, enum_name: Ident) -> Self {
        // We build the entries according to the fields type of the variant.
        match &var.fields {
            // If it is a unit variant, we return the previous vector of entries with only one entry.
            Fields::Unit => {
                let (lit, default) = match get_attr_with_args(&var.attrs, "prompt").map(take_val) {
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
                let entries = vec![Entry::from_var(enum_name, var.ident, lit, default)];
                Self { entries }
            }
            // Otherwise, we create a new vector with the entries from the fields of the variant.
            Fields::Named(FieldsNamed { named, .. }) => {
                match get_attr_with_args(&var.attrs, "prompt").map(take_val) {
                    Some(NamedAttr { fields }) => Self::new(enum_name, fields, var.ident, true),
                    None => abort_unbounds_fields(true, named),
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                match get_attr_with_args(&var.attrs, "prompt").map(take_val) {
                    Some(UnnamedAttr { fields }) => Self::new(enum_name, fields, var.ident, false),
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

impl Parse for Sp<RootParam> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Ident) {
            // The provided identifier must be either `msg` or `fmt`.
            let id = input.parse::<Ident>()?;
            let span = id.span();

            let val = match to_str!(id) {
                "msg" => {
                    input.parse::<Token![=]>()?;
                    RootParam::Msg(input.parse()?)
                }
                "fmt" => {
                    let content;
                    parenthesized!(content in input);
                    RootParam::Fmt(content.parse()?)
                }
                "nodoc" => RootParam::NoDoc,
                "case" => {
                    input.parse::<Token![=]>()?;
                    RootParam::Case(input.parse()?)
                }
                "raw" => RootParam::RawIdent,
                _ => abort_invalid_ident(id, &["msg", "fmt", "nodoc", "case"]),
            };

            Self { span, val }
        } else {
            // Else, the next token must be a string literal to represent the given message.
            let lit = input.parse::<LitStr>()?;
            let span = lit.span();
            let val = RootParam::Msg(lit);
            Self { span, val }
        })
    }
}

define_attr! {
    RootParam(sp) -> RootAttr {
        Msg(m) => msg: Option<LitStr> = None; if msg.is_none() => Some(m),
        Fmt(f) => fmt: Option<Format> = None; if fmt.is_none() => Some(f),
        Case(c) => case: Option<Case> = None; if case.is_none() => Some(c),
        NoDoc => nodoc: bool = false; if !nodoc && msg.is_none() => true,
        RawIdent => raw_ident: bool = false; if !raw_ident && msg.is_none() => true
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
        let (case, msg, fmt) = match get_attr_with_args(attrs, "prompt").map(take_val) {
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
    attrs: Vec<Attribute>,
    name: Ident,
    gens: Generics,
    variants: Punctuated<Variant, Token![,]>,
) -> TokenStream {
    if !gens.params.is_empty() {
        abort!(gens, "derive(Prompted) only supports non-generic enums.");
    }

    // The name of the library.
    let root = get_lib_root();

    set_dummy(quote! {
        impl #root::menu::Prompted for #name {
            fn from_values<H: #root::menu::Handle>(_: &mut #root::menu::Values<H>) -> #root::MenuResult<Self> {
                unimplemented!()
            }
        }

        impl #root::field::Selectable<0> for #name {
            fn select() -> #root::field::Selected<'static, Self, 0> {
                unimplemented!()
            }
        }
    });

    let data = RootData::new(name.clone(), &attrs);

    // The message and optional `Selected::format` method call, retrieved from
    // the optional attribute and the name of the enum.
    let (msg, fmt_fn) = (data.msg, data.fmt.map(|f| method_call("format", f)));

    // We map the variants into an iterator of selectable entries.
    // A variant can have multiple entries if it has fields.
    let entries = variants.into_iter().flat_map(|v| {
        Select::from_variant_with(v, data.case, name.clone())
            .entries
            .into_iter()
    });
    // We count the amount of entries to specify in the const generic argument
    // of the `Selectable` trait.
    let n = Index::from(entries.clone().count());
    // We retrieve the `Selectable::default` function expansion from the entries.
    let default_fn = get_default_fn(entries.clone());

    let fn_get_select = format_ident!("__{}_selected", name);

    quote! {
        #[allow(non_snake_case)]
        fn #fn_get_select() -> #root::field::Selected<'static, #name, #n> {
            #root::field::Selected::new(#msg, [#(#entries),*])
            #fmt_fn
            #default_fn
        }

        impl #root::menu::Prompted for #name {
            fn from_values<H: #root::menu::Handle>(vals: &mut #root::menu::Values<H>) -> #root::MenuResult<Self> {
                vals.next(#fn_get_select())
            }
        }

        impl #root::field::Selectable<#n> for #name {
            fn select() -> #root::field::Selected<'static, Self, #n> {
                #fn_get_select()
            }
        }
    }
}

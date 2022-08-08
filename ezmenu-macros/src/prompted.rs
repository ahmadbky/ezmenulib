mod promptable;
mod select;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, abort_call_site, set_dummy, ResultExt};
use quote::{quote, ToTokens};
use syn::{
    ext::IdentExt,
    parenthesized, parse,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    AngleBracketedGenericArguments, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    ExprClosure, Field, Fields, FieldsNamed, FieldsUnnamed, GenericArgument, Generics, Ident,
    LitStr, Path, PathArguments, Token, Type,
};

use crate::{
    format::Format,
    utils::{
        abort_invalid_ident, get_attr_with_args, get_first_doc, get_last_seg_of, get_lib_root,
        is_ty, method_call, split_ident_camel_case, split_ident_snake_case, take_val, to_str, Case,
        MethodCall, Sp,
    },
};

use self::{
    promptable::{Bool, RawSelectedField, Selected, Separated, Written, WrittenUntil},
    select::build_select,
};

/// Represents a parameter in the prompt attribute of a struct that contains fields.
enum RootFieldsParam {
    Case(Case),
    Fmt(Format),
    Title(LitStr),
    NoTitle,
    NoDoc,
    Raw,
}

impl Parse for RootFieldsParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Ident::peek_any) {
            let id = input.parse::<Ident>()?;
            match to_str!(id) {
                "fmt" => {
                    let content;
                    parenthesized!(content in input);
                    Self::Fmt(content.parse()?)
                }
                "case" => {
                    input.parse::<Token![=]>()?;
                    Self::Case(input.parse()?)
                }
                "title" => {
                    input.parse::<Token![=]>()?;
                    Self::Title(input.parse()?)
                }
                "no_title" => Self::NoTitle,
                "nodoc" => Self::NoDoc,
                "raw" => Self::Raw,
                _ => abort_invalid_ident(id, &["fmt", "case", "raw"]),
            }
        } else {
            Self::Title(input.parse()?)
        })
    }
}

/// Represents the attribute of a struct that contains fields.
#[derive(Default)]
struct RootFieldsAttr {
    case: Option<Case>,
    fmt: Option<Format>,
    title: Option<LitStr>,
    nodoc: bool,
    raw: bool,
    no_title: bool,
}

impl Parse for RootFieldsAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use RootFieldsParam::*;

        let mut case = None;
        let mut fmt = None;
        let mut title = None;
        let mut nodoc = false;
        let mut raw = false;
        let mut no_title = false;

        let mut vals = Punctuated::<_, Token![,]>::parse_terminated(input)?.into_iter();

        for _ in 0..4.min(vals.len()) {
            match vals.next() {
                Some(Case(c)) => case = Some(c),
                Some(Fmt(f)) => fmt = Some(f),
                Some(Title(lit)) => title = Some(lit),
                Some(NoDoc) => nodoc = true,
                Some(NoTitle) => no_title = true,
                Some(Raw) => raw = true,
                None => (),
            }
        }

        match title {
            Some(t) if !no_title => {
                abort!(t, "cannot provide a title and the `no_title` restriction")
            }
            _ => (),
        }

        Ok(Self {
            case,
            fmt,
            title,
            nodoc,
            raw,
            no_title,
        })
    }
}

impl From<&[Attribute]> for RootFieldsAttr {
    fn from(attrs: &[Attribute]) -> Self {
        get_attr_with_args(attrs, "prompt")
            .map(take_val)
            .unwrap_or_default()
    }
}

/// Represents the attribute of an unit struct.
struct RootUnitAttr {
    raw: bool,
}

impl Parse for RootUnitAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let raw = if input.peek(Ident) {
            let id = input.parse::<Ident>()?;
            if id != "raw" {
                abort_invalid_ident(id, &["raw"]);
            }
            true
        } else {
            false
        };

        Ok(Self { raw })
    }
}

/// Represents a function expression.
pub enum FunctionExpr {
    /// Provided as a closure `|x| expr`.
    Closure(ExprClosure),
    /// Provided as a path to the function.
    Func(Path),
}

impl Parse for FunctionExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![|]) {
            Self::Closure(input.parse()?)
        } else {
            Self::Func(input.parse()?)
        })
    }
}

impl ToTokens for FunctionExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FunctionExpr::Closure(c) => c.to_tokens(tokens),
            FunctionExpr::Func(f) => f.to_tokens(tokens),
        }
    }
}

/// Represents a parameter in the prompt attribute of a struct field.
enum FieldAttrParam {
    /* Every promptable */
    /// The `msg = "..."` or more simply `"..."` parameter.
    Msg(LitStr),
    /// The `fmt(...)` parameter.
    Fmt(Format),
    /// The `opt` identifier.
    Optional,
    /// The `or_default` identifier.
    OrDefault,
    /// The `case = ...` parameter.
    Case(Case),
    /// The `nodoc` identifier.
    NoDoc,
    /// The `raw` identifier.
    Raw,
    /// The `flatten` identifier.
    Flatten,

    /* Selected */
    /// The `select(...)` parameter, with its entries.
    Select(Punctuated<RawSelectedField, Token![,]>),

    /* Written/WrittenUntil/Separated */
    /// The `example = "..."` parameter.
    Example(LitStr),
    /// The `or_val("...")` parameter.
    OrVal(LitStr),
    /// The `or_env("...")` parameter.
    OrEnv(LitStr),

    /* WrittenUntil */
    /// The `until(...)` parameter.
    Until(FunctionExpr),

    /* Separated */
    /// The `sep = "...` parameter.
    Sep(LitStr),
    /// The `or_env_with("var", "sep")` parameter.
    OrEnvWithSep(LitStr, LitStr),
}

impl Parse for FieldAttrParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Ident::peek_any) {
            let id = input.parse::<Ident>()?;
            match to_str!(id) {
                "msg" => {
                    input.parse::<Token![=]>()?;
                    Self::Example(input.parse()?)
                }
                "optional" | "opt" => Self::Optional,
                "or_default" => Self::OrDefault,
                "case" => {
                    input.parse::<Token![=]>()?;
                    Self::Case(input.parse()?)
                }
                "nodoc" => Self::NoDoc,
                "raw" => Self::Raw,
                "flatten" => Self::Flatten,
                "example" => {
                    input.parse::<Token![=]>()?;
                    Self::Example(input.parse()?)
                }
                "sep" => {
                    input.parse::<Token![=]>()?;
                    Self::Sep(input.parse()?)
                }
                other => {
                    let content;
                    parenthesized!(content in input);
                    match other {
                        "fmt" => Self::Fmt(content.parse()?),
                        "select" => Self::Select(content.parse_terminated(Parse::parse)?),
                        "or" | "or_val" => Self::OrVal(content.parse()?),
                        "or_env" => Self::OrEnv(content.parse()?),
                        "or_env_with" | "env_sep" => {
                            let var = content.parse()?;
                            content.parse::<Token![,]>()?;
                            Self::OrEnvWithSep(var, content.parse()?)
                        }
                        "until" => Self::Until(content.parse()?),
                        _ => abort_invalid_ident(
                            id,
                            &[
                                "optional",
                                "opt",
                                "or_default",
                                "example",
                                "sep",
                                "fmt",
                                "select",
                                "or",
                                "or_val",
                                "or_env",
                                "or_env_with",
                                "env_sep",
                                "until",
                            ],
                        ),
                    }
                }
            }
        } else {
            Self::Msg(input.parse()?)
        })
    }
}

/// Represents the prompt attribute of a struct field.
#[derive(Default)]
struct RawFieldAttr {
    msg: Option<LitStr>,
    fmt: Option<Format>,
    opt: bool,
    or_default: bool,
    nodoc: bool,
    case: Option<Case>,
    raw: bool,
    flatten: bool,

    select: Option<Punctuated<RawSelectedField, Token![,]>>,

    example: Option<LitStr>,
    default_val: Option<LitStr>,
    default_env: Option<LitStr>,

    until: Option<FunctionExpr>,

    sep: Option<LitStr>,
    // (environment variable, separator)
    default_env_with_sep: Option<(LitStr, LitStr)>,
}

impl Parse for RawFieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use FieldAttrParam::*;

        let mut msg = None;
        let mut fmt = None;
        let mut opt = false;
        let mut or_default = false;
        let mut case = None;
        let mut nodoc = false;
        let mut raw = false;
        let mut flatten = false;

        let mut select = None;

        let mut example = None;
        let mut default_val = None;
        let mut default_env = None;

        let mut until = None;

        let mut sep = None;
        let mut default_env_with_sep = None;

        let mut vals = Punctuated::<_, Token![,]>::parse_terminated(input)?.into_iter();

        // The attribute can have maximum 9 values if provided as "many"
        // msg ; fmt ; optional ; or_default ; example ;
        // default_val ; default_env ; sep ; default_env_with_sep
        for _ in 0..9.min(vals.len()) {
            match vals.next() {
                Some(Msg(m)) => msg = Some(m),
                Some(Fmt(f)) => fmt = Some(f),
                Some(Optional) => opt = true,
                Some(OrDefault) => or_default = true,
                Some(Case(c)) => case = Some(c),
                Some(NoDoc) => nodoc = true,
                Some(Raw) => raw = true,
                Some(Flatten) => flatten = true,

                Some(Select(sel)) => select = Some(sel),

                Some(Example(e)) => example = Some(e),
                Some(OrVal(v)) => default_val = Some(v),
                Some(OrEnv(v)) => default_env = Some(v),

                Some(Until(f)) => until = Some(f),

                Some(Sep(s)) => sep = Some(s),
                Some(OrEnvWithSep(v, s)) => default_env_with_sep = Some((v, s)),

                None => (),
            }
        }

        Ok(Self {
            msg,
            fmt,
            opt,
            or_default,
            case,
            nodoc,
            raw,
            flatten,

            select,

            example,
            default_val,
            default_env,

            until,
            sep,
            default_env_with_sep,
        })
    }
}

/// Returns the nested type inside the chevrons
///
/// &`Option<T>` --> Some(&`T`)
fn get_nested_type(ty: &Type) -> Option<&Type> {
    let nested = get_last_seg_of(ty)
        .filter(|s| s.ident == "Option")
        .and_then(|s| {
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
                &s.arguments
            {
                Some(args.first()).flatten()
            } else {
                None
            }
        });

    match nested {
        Some(GenericArgument::Type(ty)) => Some(ty),
        _ => None,
    }
}

/// Represents the type of a prompt used with the Values struct methods.
enum PromptKind {
    /// Values::next method, needing a `?` at the call.
    Next,
    /// Values::next_or_default method, without a `?` at the call.
    NextOrDefault,
    /// Values::next_optional method, needing a `?` at the call,
    /// and unwrapping the nested output type from the `Option<T>` type.
    NextOptional,
}

impl PromptKind {
    /// Returns the "method called" version of the prompt kind, from the
    /// output type and the value used as argument.
    ///
    /// It unwraps the nested type inside the `Option<...>` if so.
    fn call_for<T>(self, ty: &Type, val: T) -> MethodCall<T> {
        let s = match self {
            Self::Next => "next",
            Self::NextOrDefault => "next_or_default",
            Self::NextOptional => "next_optional",
        };

        let ty = match self {
            Self::NextOptional => get_nested_type(ty).unwrap_or(ty),
            _ => ty,
        };

        let out = method_call(s, val)
            .with_span(ty.span())
            .with_generics(vec![ty.clone(), parse("_".parse().unwrap()).unwrap()]);

        if let Self::NextOrDefault = self {
            out
        } else {
            out.with_question()
        }
    }
}

/// Represents a promptable provided as argument of the prompt call.
///
/// In the library, they corresponds to the types that implements the Promptable trait.
enum Promptable {
    /// The Selected type.
    Selected(Selected),
    /// The Written type.
    Written(Written),
    /// The WrittenUntil type.
    WrittenUntil(WrittenUntil),
    /// The Separated type.
    Separated(Separated),
    /// The Bool type.
    Bool(Bool),
}

impl ToTokens for Promptable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Selected(s) => s.to_tokens(tokens),
            Self::Written(w) => w.to_tokens(tokens),
            Self::WrittenUntil(w) => w.to_tokens(tokens),
            Self::Separated(s) => s.to_tokens(tokens),
            Self::Bool(b) => b.to_tokens(tokens),
        }
    }
}

/// Represents the prompt call of a struct field.
enum FieldPrompt {
    /// The call is flatten, meaning the output type already implements `Prompted` trait,
    /// so we can construct it from the `Prompted::from_values` method.
    Flatten,
    /// The basic prompt, expanded to `vals.next(Promptable)` for example.
    Basic(MethodCall<Promptable>),
}

impl FieldPrompt {
    /// Returns the prompt call of the field from its prompt attribute and the message of the prompt.
    ///
    /// The message retrieval depends on the field type (named/unnamed).
    fn new(attr: Sp<RawFieldAttr>, field: Field, msg: String) -> Self {
        let fmt = attr.val.fmt.map(|f| method_call("format", f));
        let kind = match (attr.val.opt, attr.val.or_default) {
            (true, true) => abort_opt_or_default(attr.span),
            (true, false) => PromptKind::NextOptional,
            (false, true) if is_ty(&field.ty, "Option") => abort_opt_or_default(attr.span),
            (false, true) => PromptKind::NextOrDefault,
            (false, false) if is_ty(&field.ty, "Option") => PromptKind::NextOptional,
            (false, false) => PromptKind::Next,
        };
        let example = attr.val.example.map(|e| method_call("example", e));
        let default_val = attr.val.default_val.map(|e| method_call("default_val", e));
        let default_env = attr.val.default_env.map(|e| method_call("default_env", e));

        if let Some(entries) = attr.val.select {
            // Selected promptable

            if example.is_some()
                || default_val.is_some()
                || default_env.is_some()
                || attr.val.until.is_some()
                || attr.val.sep.is_some()
                || attr.val.default_env_with_sep.is_some()
            {
                abort!(
                    attr.span,
                    "cannot define field as selectable and provide attributes as a written field"
                );
            }

            let prompt = Promptable::Selected(Selected::new(msg, fmt, entries).unwrap_or_abort());
            Self::Basic(kind.call_for(&field.ty, prompt))
        } else if attr.val.flatten {
            // Flattened prompt, we call `Prompted::from_values` method for this field
            Self::Flatten
        } else {
            // "Writtens" promptable

            let w = Written {
                msg,
                fmt,
                example,
                default_val,
                default_env,
            };

            let prompt = if let Some(til) = attr.val.until {
                // WrittenUntil promptable

                if attr.val.default_env_with_sep.is_some() {
                    abort!(
                        attr.span,
                        "cannot provide a separator for environment variable and an `until` function"
                    )
                }
                Promptable::WrittenUntil(WrittenUntil { w, til })
            } else if let Some(sep) = attr.val.sep {
                // Separated promptable

                let env_sep = attr
                    .val
                    .default_env_with_sep
                    .map(|(var, sep)| method_call("default_env_with", quote!(#var, #sep)));
                Promptable::Separated(Separated { w, sep, env_sep })
            } else if is_ty(&field.ty, "bool") {
                // Bool promptable
                Promptable::Bool(Bool { w })
            } else {
                // Written promptable

                Promptable::Written(w)
            };

            Self::Basic(kind.call_for(&field.ty, prompt))
        }
    }
}

impl ToTokens for FieldPrompt {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldPrompt::Flatten => {
                let root = get_lib_root();
                quote!(#root::menu::Prompted::from_values(vals)?).to_tokens(tokens)
            }
            FieldPrompt::Basic(call) => quote!(vals #call).to_tokens(tokens),
        }
    }
}

/// Util function used to emit an error emphasizing that the field is both optional and
/// returns the `Default::default` trait implementation value.
fn abort_opt_or_default(span: Span) -> ! {
    abort!(
        span,
        "cannot define field as both optional and using `impl Default` value"
    );
}

/// Returns the field prompt from the field itself and the global case of the struct if provided.
fn get_field_prompt(field: Field, case: Option<&Case>) -> FieldPrompt {
    let attr: Sp<RawFieldAttr> = get_attr_with_args(&field.attrs, "prompt").unwrap_or_default();

    let msg = attr
        .val
        .msg
        .as_ref()
        .map(LitStr::value)
        .or_else(|| {
            if attr.val.nodoc {
                None
            } else {
                get_first_doc(&field.attrs)
            }
        })
        .unwrap_or_else(|| {
            if let Some(name) = field.ident.as_ref() {
                if attr.val.raw {
                    name.to_string()
                } else {
                    split_ident_snake_case(name)
                }
            } else {
                abort!(
                    field,
                    "this field must contain at least a `#[prompt(msg = \"...\")]` attribute"
                )
            }
        });

    let msg = match attr.val.case.as_ref().or(case) {
        Some(c) => c.map(msg),
        None => msg,
    };

    FieldPrompt::new(attr, field, msg)
}

/// Represents an unnamed field of a struct.
struct UnnamedField {
    prompt: FieldPrompt,
}

impl UnnamedField {
    /// Returns the unnamed field with the optional case of the struct attribute if provided.
    fn new(field: Field, case: Option<&Case>) -> Self {
        let prompt = get_field_prompt(field, case);
        Self { prompt }
    }
}

impl ToTokens for UnnamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.prompt.to_tokens(tokens);
    }
}

/// Represents the construction expansion of the unnamed struct.
struct UnnamedInit {
    values: Punctuated<UnnamedField, Token![,]>,
}

impl UnnamedInit {
    fn new(unnamed: Punctuated<Field, Token![,]>, case: Option<&Case>) -> Self {
        let values = unnamed
            .into_iter()
            .map(|f| UnnamedField::new(f, case))
            .collect();
        Self { values }
    }
}

impl ToTokens for UnnamedInit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let values = &self.values;
        quote!(Self(#values)).to_tokens(tokens);
    }
}

/// Represents a named field of a struct.
struct NamedField {
    name: Ident,
    prompt: FieldPrompt,
}

impl NamedField {
    /// Returns the named field with the optional case of the struct attribute if provided.
    fn new(field: Field, case: Option<&Case>) -> Self {
        let name = field.ident.clone().unwrap();
        let prompt = get_field_prompt(field, case);
        Self { name, prompt }
    }
}

impl ToTokens for NamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let call = &self.prompt;
        quote!(#name: #call).to_tokens(tokens);
    }
}

/// Represents the construction expansion of the named struct.
struct NamedInit {
    fields: Punctuated<NamedField, Token![,]>,
}

impl NamedInit {
    fn new(fields: Punctuated<Field, Token![,]>, case: Option<&Case>) -> Self {
        let fields = fields
            .into_iter()
            .map(|f| NamedField::new(f, case))
            .collect();
        Self { fields }
    }
}

impl ToTokens for NamedInit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields = &self.fields;
        quote!(Self { #fields }).to_tokens(tokens);
    }
}

/// Maps the name of the unit struct to its lowercase string representation.
///
/// It splits the identifier of the struct by default unless the `raw` parameter has been
/// provided in the prompt attribute.
fn map_unit_ident(attrs: &[Attribute], name: &Ident) -> String {
    match get_attr_with_args(attrs, "prompt").map(take_val) {
        Some(RootUnitAttr { raw: true }) => name.to_string(),
        _ => split_ident_camel_case(name),
    }
    .to_lowercase()
}

/// Expands the `derive(Prompted)` macro for an unit struct.
///
/// This expansion consists of the implementation of the FromStr trait on this struct.
fn build_unit_struct(attrs: Vec<Attribute>, name: Ident) -> TokenStream {
    let low_name = map_unit_ident(&attrs, &name);
    let err_msg = format!("failed to parse to {name} struct");

    quote! {
        impl ::std::str::FromStr for #name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    #low_name => Ok(Self),
                    _ => Err(#err_msg.to_owned()),
                }
            }
        }
    }
}

/// Returns the TokenStream of the `writeln!(...)` instruction to display a message
/// before retrieving the values.
///
/// It uses, in order, the message provided in the prompt attribute, or the doc comment message if the
/// `nodoc` parameter hasn't been provided, or the splitted version of the struct name
/// if the `raw` parameter hasn't been provided, or the struct name itself.
///
/// It maps the optional case to the message.
fn disp_title_ts(data: &RootFieldsAttr, attrs: &[Attribute], name: &Ident) -> TokenStream {
    let name = data
        .title
        .as_ref()
        .map(LitStr::value)
        .or_else(|| {
            if data.nodoc {
                None
            } else {
                get_first_doc(attrs)
            }
        })
        .unwrap_or_else(|| {
            if data.raw {
                name.to_string()
            } else {
                split_ident_camel_case(name)
            }
        });

    let name = match data.case.as_ref() {
        Some(c) => c.map(name),
        None => name,
    };

    quote!(writeln!(vals.handle, #name)?;)
}

/// Returns the TokenStream of the struct construction.
///
/// This function is called after checking that the struct isn't an unit struct.
fn construct_ts(case: Option<&Case>, fields: Fields) -> TokenStream {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => NamedInit::new(named, case).into_token_stream(),
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            UnnamedInit::new(unnamed, case).into_token_stream()
        }
        _ => unreachable!(),
    }
}

/// Expands the `derive(Prompted)` macro on a struct that contains fields.
fn build_fields_struct(
    attrs: Vec<Attribute>,
    name: Ident,
    gens: Generics,
    fields: Fields,
) -> TokenStream {
    let where_clause = gens.where_clause.as_ref();
    // The name of the library.
    let root = get_lib_root();

    set_dummy(quote! {
        impl #gens #root::menu::Prompted for #name #where_clause {
            fn from_values<H: #root::menu::Handle>(_vals: &mut #root::menu::Values<H>) -> #root::MenuResult<Self> {
                unimplemented!()
            }
        }
    });

    let data = RootFieldsAttr::from(attrs.as_ref());

    let fmt_fn = data
        .fmt
        .as_ref()
        .map(|f| method_call("format", f))
        .into_token_stream();
    let disp_title = if data.no_title {
        None
    } else {
        Some(disp_title_ts(&data, &attrs, &name))
    };

    let init = construct_ts(data.case.as_ref(), fields);

    quote! {
        impl #gens #root::menu::Prompted for #name #where_clause {
            fn try_prompt_with<H: #root::menu::Handle>(handle: H) -> #root::MenuResult<Self> {
                Self::from_values(&mut #root::menu::Values::from_handle(handle) #fmt_fn)
            }

            fn from_values<H: #root::menu::Handle>(vals: &mut #root::menu::Values<H>) -> #root::MenuResult<Self> {
                #disp_title
                Ok(#init)
            }
        }
    }
}

/// Expands the `derive(Prompted)` macro for a struct.
fn build_struct(attrs: Vec<Attribute>, name: Ident, gens: Generics, fields: Fields) -> TokenStream {
    match fields {
        Fields::Unit => build_unit_struct(attrs, name),
        other => build_fields_struct(attrs, name, gens, other),
    }
}

/// Expands the `derive(Prompted)` macro.
pub fn build_prompted(input: DeriveInput) -> TokenStream {
    match input.data {
        Data::Enum(DataEnum { variants, .. }) => {
            build_select(input.attrs, input.ident, input.generics, variants)
        }
        Data::Struct(DataStruct { fields, .. }) => {
            build_struct(input.attrs, input.ident, input.generics, fields)
        }
        _ => abort_call_site!("derive(Prompted) only supports enums and structs"),
    }
}

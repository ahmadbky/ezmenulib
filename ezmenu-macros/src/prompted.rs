mod promptable;
mod select;

use proc_macro2::{Punct, Spacing, Span, TokenStream};
use proc_macro_error::{abort, abort_call_site, set_dummy, ResultExt};
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{
    ext::IdentExt,
    parenthesized, parse,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    AngleBracketedGenericArguments, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    ExprClosure, Field, Fields, FieldsNamed, FieldsUnnamed, GenericArgument, Generics, Ident,
    LitStr, Path, PathArguments, Token, Type, TypeInfer, TypePath,
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

pub enum FunctionExpr {
    Closure(ExprClosure),
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

enum FieldAttrParam {
    // Every promptable
    Msg(LitStr),
    Fmt(Format),
    Optional,
    OrDefault,
    NoDoc,
    Raw,

    // Selected
    Select(Punctuated<RawSelectedField, Token![,]>),

    // Written/WrittenUntil/Separated
    Example(LitStr),
    OrVal(LitStr),
    OrEnv(LitStr),

    // WrittenUntil
    Until(FunctionExpr),

    // Separated
    Sep(LitStr),
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
                "nodoc" => Self::NoDoc,
                "raw" => Self::Raw,
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

#[derive(Default)]
struct RawFieldAttr {
    msg: Option<LitStr>,
    fmt: Option<Format>,
    opt: bool,
    or_default: bool,
    nodoc: bool,
    raw: bool,

    select: Option<Punctuated<RawSelectedField, Token![,]>>,

    example: Option<LitStr>,
    default_val: Option<LitStr>,
    default_env: Option<LitStr>,

    until: Option<FunctionExpr>,

    sep: Option<LitStr>,
    // environment variable, separator
    default_env_with_sep: Option<(LitStr, LitStr)>,
}

impl Parse for RawFieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use FieldAttrParam::*;

        let mut msg = None;
        let mut fmt = None;
        let mut opt = false;
        let mut or_default = false;
        let mut nodoc = false;
        let mut raw = false;

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
                Some(NoDoc) => nodoc = true,
                Some(Raw) => raw = true,

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
            nodoc,
            raw,

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

enum PromptKind {
    Next,
    NextOrDefault,
    NextOptional,
}

impl PromptKind {
    fn call_for<T>(self, ty: &Type, val: T) -> MethodCall<T> {
        let s = match self {
            Self::Next => "next",
            Self::NextOrDefault => "next_or_default",
            Self::NextOptional => "next_optional",
        };

        let ty = match self {
            Self::NextOptional => {
                let nested = get_last_seg_of(&ty)
                    .filter(|s| s.ident == "Option")
                    .and_then(|s| {
                        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            args,
                            ..
                        }) = &s.arguments
                        {
                            Some(args.first()).flatten()
                        } else {
                            None
                        }
                    });

                if let Some(GenericArgument::Type(ty)) = nested {
                    ty
                } else {
                    ty
                }
            }
            _ => ty,
        };

        let out = method_call(s, val)
            .with_span(ty.span())
            .with_generics(vec![ty.clone(), parse("_".parse().unwrap()).unwrap()]);
        let out = if let Self::NextOrDefault = self {
            out
        } else {
            out.with_question()
        };

        out
    }
}

enum Promptable {
    Selected(Selected),
    Written(Written),
    WrittenUntil(WrittenUntil),
    Separated(Separated),
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

fn abort_opt_or_default(span: Span) -> ! {
    abort!(
        span,
        "cannot define field as both optional and using `impl Default` value"
    );
}

struct UnnamedField {
    call: MethodCall<Promptable>,
}

impl From<Field> for UnnamedField {
    fn from(field: Field) -> Self {
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
                abort!(
                    field,
                    "this field must contain at least a `#[prompt(msg = \"...\")]` attribute"
                )
            });

        let call = get_prompt_call(attr, field, msg);
        Self { call }
    }
}

fn get_prompt_call(attr: Sp<RawFieldAttr>, field: Field, msg: String) -> MethodCall<Promptable> {
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

    let promptable = if let Some(entries) = attr.val.select {
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

        Promptable::Selected(Selected::new(msg, fmt, entries).unwrap_or_abort())
    } else {
        let w = Written {
            msg,
            fmt,
            example,
            default_val,
            default_env,
        };

        if let Some(til) = attr.val.until {
            if attr.val.default_env_with_sep.is_some() {
                abort!(
                    attr.span,
                    "cannot provide a separator for environment variable and an `until` function"
                )
            }
            Promptable::WrittenUntil(WrittenUntil { w, til })
        } else if let Some(sep) = attr.val.sep {
            let env_sep = attr
                .val
                .default_env_with_sep
                .map(|(var, sep)| method_call("default_env_with", quote!(#var, #sep)));
            Promptable::Separated(Separated { w, sep, env_sep })
        } else if is_ty(&field.ty, "bool") {
            Promptable::Bool(Bool { w })
        } else {
            Promptable::Written(w)
        }
    };

    kind.call_for(&field.ty, promptable)
}

impl ToTokens for UnnamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let call = &self.call;
        quote!(vals #call).to_tokens(tokens);
    }
}

struct UnnamedInit {
    values: Punctuated<UnnamedField, Token![,]>,
}

impl From<Punctuated<Field, Token![,]>> for UnnamedInit {
    fn from(unnamed: Punctuated<Field, Token![,]>) -> Self {
        let values = unnamed.into_iter().map(UnnamedField::from).collect();
        Self { values }
    }
}

impl ToTokens for UnnamedInit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let values = &self.values;
        quote!(Self(#values)).to_tokens(tokens);
    }
}

struct NamedField {
    name: Ident,
    call: MethodCall<Promptable>,
}

impl NamedField {
    fn new(field: Field, case: Option<&Case>) -> Self {
        let attr: Sp<RawFieldAttr> = get_attr_with_args(&field.attrs, "prompt").unwrap_or_default();
        let ident = field.ident.as_ref().unwrap();

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
                if attr.val.raw {
                    ident.to_string()
                } else {
                    split_ident_snake_case(&ident)
                }
            });
        let msg = match case {
            Some(c) => c.map(msg),
            None => msg,
        };

        let name = ident.clone();
        let call = get_prompt_call(attr, field, msg);

        Self { name, call }
    }
}

impl ToTokens for NamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let call = &self.call;
        quote!(#name: vals #call).to_tokens(tokens);
    }
}

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

fn map_unit_ident(attrs: &[Attribute], name: &Ident) -> String {
    match get_attr_with_args(attrs, "prompt").map(take_val) {
        Some(RootUnitAttr { raw: true }) => name.to_string(),
        _ => split_ident_camel_case(name),
    }
    .to_lowercase()
}

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
    quote!(writeln!(handle, #name)?;)
}

fn construct_ts(case: Option<&Case>, fields: Fields) -> TokenStream {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => NamedInit::new(named, case).into_token_stream(),
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            UnnamedInit::from(unnamed).into_token_stream()
        }
        _ => unreachable!(),
    }
}

fn build_fields_struct(
    attrs: Vec<Attribute>,
    name: Ident,
    gens: Generics,
    fields: Fields,
) -> TokenStream {
    let where_clause = gens.where_clause.as_ref();
    let root = get_lib_root();

    set_dummy(quote! {
        impl #gens #root::menu::Prompted for #name #where_clause {
            fn try_prompt_with<H: #root::menu::Handle>(handle: H) -> #root::MenuResult<Self> {
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
            fn try_prompt_with<H: #root::menu::Handle>(mut handle: H) -> #root::MenuResult<Self> {
                #disp_title
                let mut vals = #root::menu::Values::from_handle(handle)
                    #fmt_fn;
                Ok(#init)
            }
        }
    }
}

fn build_struct(attrs: Vec<Attribute>, name: Ident, gens: Generics, fields: Fields) -> TokenStream {
    match fields {
        Fields::Unit => build_unit_struct(attrs, name),
        other => build_fields_struct(attrs, name, gens, other),
    }
}

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

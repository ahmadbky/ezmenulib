pub(crate) mod promptable;
mod select;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, abort_call_site, set_dummy, ResultExt};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Data, DataEnum, DataStruct, DeriveInput, ExprClosure, Field, Fields, FieldsNamed,
    FieldsUnnamed, GenericArgument, Generics, Ident, LitStr, Pat, PatIdent, PatStruct,
    PatTupleStruct, PatType, Path, Token, Type,
};

use crate::{
    format::Format,
    generics::check_for_bound,
    kw::define_attr,
    utils::{
        get_attr_with_args, get_first_doc, get_last_seg_of_ty, get_lib_root, get_lib_root_spanned,
        get_nested_args, get_ty_ident, is_ty, method_call, method_call_empty,
        split_ident_camel_case, split_ident_snake_case, take_val, Case, MethodCall, Sp,
    },
};

use self::{
    promptable::{Bool, RawSelectedField, Selected, Separated, Written, WrittenUntil},
    select::build_select,
};

define_attr! {
    RootFieldsAttr {
        case: Option<Case>,
        fmt: Option<Format>,
        title: Option<LitStr>,
        nodoc: bool,
        raw: bool,
        no_title: bool,
    }
}

impl From<&[Attribute]> for RootFieldsAttr {
    fn from(attrs: &[Attribute]) -> Self {
        get_attr_with_args(attrs, "prompt")
            .map(take_val)
            .unwrap_or_default()
    }
}

define_attr! {
    RootUnitAttr {
        raw: bool,
    }
}

/// Represents a function expression.
#[derive(Debug)]
pub(crate) enum FunctionExpr {
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

define_attr! {
    RawFieldAttr {
        msg: Option<LitStr>,
        fmt: Option<Format>,
        optional: bool,
        or_default: bool; without optional,
        case: Option<Case>,
        nodoc: bool,
        raw: bool; without msg,
        flatten: bool; without
            msg; fmt; optional; or_default; case; nodoc; raw; select; example;
            or_val; or_env; until; sep; or_env_with,

        select: Option<Punctuated<RawSelectedField, Token![,]>>; without
            msg; nodoc; raw; example; or_val; or_env; until; sep; or_env_with,

        example: Option<LitStr>,
        or_val: Option<LitStr>,
        or_env: Option<LitStr>,

        until: Option<FunctionExpr>; without sep; or_env_with,

        sep: Option<LitStr>,
        or_env_with: Option<(LitStr, LitStr)>,

        basic_example: bool; without until; sep; or_env_with,
    }
}

/// Returns the nested type inside the chevrons of an `Option<T>` type.
///
/// &`Option<T>` --> Some(&`T`)
fn get_nested_type(ty: &Type) -> Option<&Type> {
    let nested = get_last_seg_of_ty(ty)
        .filter(|s| s.ident == "Option")
        .and_then(get_nested_args)
        .and_then(Punctuated::first);

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
            .with_generics(vec![ty.clone(), parse_quote!(_)]);

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
    Basic(Box<MethodCall<Promptable>>),
}

impl FieldPrompt {
    /// Returns the prompt call of the field from its prompt attribute and the message of the prompt.
    ///
    /// The message retrieval depends on the field type (named/unnamed).
    fn new(
        attr: Sp<RawFieldAttr>,
        field: Field,
        msg: String,
        gens: &mut AugmentedGenerics,
    ) -> Self {
        let fmt = attr.val.fmt.map(|f| method_call("format", f));
        let kind = match (attr.val.opt, attr.val.or_default) {
            (true, true) => unreachable!("assert !(opt && or_default)"),
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
            let prompt = Promptable::Selected(Selected::new(msg, fmt, entries).unwrap_or_abort());
            Self::Basic(Box::new(kind.call_for(&field.ty, prompt)))
        } else if attr.val.flatten {
            // Flattened prompt, we call `Prompted::from_values` method for this field
            if let Some(id) = get_ty_ident(&field.ty) {
                let root = get_lib_root();
                gens.check_for_bound(id, quote!(#root::menu::Prompted));
            }
            Self::Flatten
        } else {
            // "Writtens" promptable

            if let Some(id) = get_ty_ident(&field.ty) {
                gens.check_for_bound(id, quote!(::std::str::FromStr));
            }

            let w = Written {
                msg,
                fmt,
                example,
                default_val,
                default_env,
            };

            let prompt = if let Some(til) = attr.val.until {
                // WrittenUntil promptable
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

            Self::Basic(Box::new(kind.call_for(&field.ty, prompt)))
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
fn get_field_prompt(
    field: Field,
    case: Option<&Case>,
    gens: &mut AugmentedGenerics,
) -> FieldPrompt {
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

    FieldPrompt::new(attr, field, msg, gens)
}

/// Represents an unnamed field of a struct.
struct UnnamedField {
    prompt: FieldPrompt,
}

impl UnnamedField {
    /// Returns the unnamed field with the optional case of the struct attribute if provided.
    fn new(field: Field, case: Option<&Case>, gens: &mut AugmentedGenerics) -> Self {
        let prompt = get_field_prompt(field, case, gens);
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
    fn new(
        unnamed: Punctuated<Field, Token![,]>,
        case: Option<&Case>,
        gens: &mut AugmentedGenerics,
    ) -> Self {
        let values = unnamed
            .into_iter()
            .map(|f| UnnamedField::new(f, case, gens))
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
    fn new(field: Field, case: Option<&Case>, gens: &mut AugmentedGenerics) -> Self {
        let name = field
            .ident
            .clone()
            .expect("called NamedField::new on an unnamed field");
        let prompt = get_field_prompt(field, case, gens);
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
    fn new(
        fields: Punctuated<Field, Token![,]>,
        case: Option<&Case>,
        gens: &mut AugmentedGenerics,
    ) -> Self {
        let fields = fields
            .into_iter()
            .map(|f| NamedField::new(f, case, gens))
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
fn build_unit_struct(attrs: Vec<Attribute>, name: Ident, gens: AugmentedGenerics) -> TokenStream {
    let low_name = map_unit_ident(&attrs, &name);
    let err_msg = format!("failed to parse to {name} struct");

    gens.impl_for(
        quote!(::std::str::FromStr),
        &name,
        quote! {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    #low_name => Ok(Self),
                    _ => Err(#err_msg.to_owned()),
                }
            }
        },
    )
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
fn construct_ts(case: Option<&Case>, fields: Fields, gens: &mut AugmentedGenerics) -> TokenStream {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            NamedInit::new(named, case, gens).into_token_stream()
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            UnnamedInit::new(unnamed, case, gens).into_token_stream()
        }
        _ => unreachable!(),
    }
}

/// Expands the `derive(Prompted)` macro on a struct that contains fields.
fn build_fields_struct(
    attrs: Vec<Attribute>,
    name: Ident,
    mut gens: AugmentedGenerics,
    fields: Fields,
) -> TokenStream {
    // The name of the library.
    let root = get_lib_root();

    set_dummy(gens.impl_for(quote!(#root::menu::Prompted), &name, quote! {
        fn from_values<H: #root::menu::Handle>(_: &mut #root::menu::Values<H>) -> #root::MenuResult<Self> {
            unimplemented!()
        }
    }));

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

    let init = construct_ts(data.case.as_ref(), fields, &mut gens);

    gens.impl_for(quote!(#root::menu::Prompted), &name, quote! {
        fn try_prompt_with<H: #root::menu::Handle>(handle: H) -> #root::MenuResult<Self> {
            Self::from_values(&mut #root::menu::Values::from_handle(handle) #fmt_fn)
        }

        fn from_values<H: #root::menu::Handle>(vals: &mut #root::menu::Values<H>) -> #root::MenuResult<Self> {
            #disp_title
            Ok(#init)
        }
    })
}

/// Expands the `derive(Prompted)` macro for a struct.
fn build_struct(attrs: Vec<Attribute>, name: Ident, gens: Generics, fields: Fields) -> TokenStream {
    let gens = AugmentedGenerics::from(gens);
    match fields {
        Fields::Unit => build_unit_struct(attrs, name, gens),
        other => build_fields_struct(attrs, name, gens, other),
    }
}

/// Expands the `derive(Prompted)` macro.
pub(crate) fn build_prompted(input: DeriveInput) -> TokenStream {
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

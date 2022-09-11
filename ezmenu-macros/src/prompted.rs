//! Module that manages the expansion of the Prompted trait.
//!
//! If the item is an enum, the expansion will be managed by the [`prompted::select`] submodule.

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
    pretend::pretend_used,
    utils::{
        get_attr_with_args, get_first_doc, get_last_seg_of_ty, get_lib_root, get_lib_root_spanned,
        get_nested_args, get_ty_ident, is_ty, method_call, method_call_empty,
        split_ident_camel_case, split_ident_snake_case, take_val, wrap_in_const, Case, MethodCall,
        Sp,
    },
};

use self::{
    promptable::{Bool, Password, RawSelectedField, Selected, Separated, Until, Written},
    select::build_select,
};

define_attr! {
    /// Represents the prompted attribute of a struct that contains named or unnamed fields.
    RootFieldsAttr {
        case: Option<Case>,
        fmt: Option<Format>,
        title: Option<LitStr>,
        nodoc: bool,
        raw: bool; without title,
        no_title: bool; without title,
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
    /// Represents the prompted attribute of an unit struct.
    RootUnitAttr {
        raw: bool,
    }
}

/// Represents a function expression.
#[derive(Clone, Debug)]
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
    /// Represents the raw prompted attribute of a named or unnamed struct field.
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
            or_val; or_env; until; sep; or_env_with; basic_example; password,

        select: Option<Punctuated<RawSelectedField, Token![,]>>; without
            msg; nodoc; raw; example; or_val; or_env; sep; or_env_with; basic_example; password;
            /* [1] this isn't mandatory but is placed for more sense */ until,

        example: Option<LitStr>,
        or_val: Option<LitStr>,
        or_env: Option<LitStr>,

        until: Option<FunctionExpr>,

        sep: Option<LitStr>,
        or_env_with: Option<(LitStr, LitStr)>,

        basic_example: bool; without sep; or_env_with; /* same as [1] */ until,

        password: bool; without example; or_val; or_env; sep; or_env_with; basic_example,
    }
}

/// Returns the nested type inside the chevrons of an `Option<T>` type.
///
/// &`Option<T>` --> Some(&`T`)
fn try_get_nested_type(ty: &Type) -> &Type {
    let nested = get_last_seg_of_ty(ty)
        .filter(|s| s.ident == "Option")
        .and_then(get_nested_args)
        .and_then(Punctuated::first);

    match nested {
        Some(GenericArgument::Type(nested)) => nested,
        _ => ty,
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
    fn call_for(self, ty: &Type, val: Promptable) -> Box<MethodCall<Promptable>> {
        let s = match self {
            Self::Next => "next",
            Self::NextOrDefault => "next_or_default",
            Self::NextOptional => "next_optional",
        };

        let ty = match self {
            Self::NextOptional => try_get_nested_type(ty),
            _ => ty,
        };

        let out = method_call(s, val)
            .with_span(ty.span())
            .with_generics(vec![ty.clone(), parse_quote!(_)]);

        Box::new(match self {
            Self::NextOrDefault => out,
            _ => out.with_question(),
        })
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
    Until(Until),
    /// The Separated type.
    Separated(Separated),
    /// The Bool type.
    Bool(Bool),
    /// The Password type.
    Password(Password),
}

impl Promptable {
    /// Returns the promptable corresponding to the field and its context.
    ///
    /// This method must be called after checking that it must not be the `until` promptable
    /// at this point.
    ///
    /// # Arguments
    ///
    /// * ty: The type of the field.
    /// * w: We use the `Written` type as a container of the information we need,
    /// for example the message.
    /// * attr: The prompted attribute of the field.
    /// * gens: The generics of the field struct, used to insert new trait bounds if we need to.
    fn from_not_until(ty: &Type, w: Written, attr: RawFieldAttr, gens: &mut Generics) -> Self {
        if let Some(entries) = attr.select {
            Promptable::Selected(Selected::new(w.msg, w.fmt, entries).unwrap_or_abort())
        } else if attr.password {
            Promptable::Password(Password {
                msg: w.msg,
                fmt: w.fmt,
            })
        } else if is_ty(ty, "bool") {
            let basic_example = attr
                .basic_example
                .then(|| method_call_empty("with_basic_example"));
            Promptable::Bool(Bool { w, basic_example })
        } else {
            // At this point, the promptable must either be `Separated` or `Written`.

            if let Some(id) = get_ty_ident(ty) {
                check_for_bound(gens, id, quote!(::core::str::FromStr));
            }

            if let Some(sep) = attr.sep {
                let env_sep = attr.or_env_with.map(|(var, sep)| {
                    MethodCall::new(
                        Ident::new("default_env_with", Span::call_site()),
                        parse_quote!(#var, #sep),
                    )
                });
                Promptable::Separated(Separated { w, sep, env_sep })
            } else {
                Promptable::Written(w)
            }
        }
    }
}

impl ToTokens for Promptable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Selected(s) => s.to_tokens(tokens),
            Self::Written(w) => w.to_tokens(tokens),
            Self::Until(u) => u.to_tokens(tokens),
            Self::Separated(s) => s.to_tokens(tokens),
            Self::Bool(b) => b.to_tokens(tokens),
            Self::Password(p) => p.to_tokens(tokens),
        }
    }
}

/// Function used to insert the type of the argument of a closure, to avoid redundance.
///
/// This is used to avoid the user to reprecise the type of the argument in the closure,
/// while it is already specified by the type of the field.
fn insert_type_for(ExprClosure { inputs, .. }: &mut ExprClosure, ty: &Type) {
    fn insert_type(inputs: &mut Punctuated<Pat, Token![,]>, ty: &Type) {
        let pat = Box::new(inputs[0].clone());
        let ty = try_get_nested_type(ty);
        inputs[0] = Pat::Type(PatType {
            attrs: vec![],
            pat,
            colon_token: Token![:](Span::call_site()),
            ty: Box::new(parse_quote!(&#ty)),
        });
    }

    match inputs.first() {
        Some(Pat::Ident(PatIdent { .. })) => insert_type(inputs, ty),
        Some(
            Pat::Struct(PatStruct { path, .. }) | Pat::TupleStruct(PatTupleStruct { path, .. }),
        ) => insert_type(inputs, &parse_quote!(#path)),
        _ => (),
    }
}

macro_rules! map_method_call {
    ($attr:ident: $( $field:ident $msg:expr ),*) => {$(
        let $field = $attr.val.$field.clone().map(|v| method_call($msg, v));
    )*};
}

/// Represents the prompt call of a struct field.
enum FieldPrompt {
    /// The call is flatten, meaning the output type already implements `Prompted` trait,
    /// so we can construct it from the `Prompted::from_values` method.
    Flatten(Span),
    /// The basic prompt, expanded to `vals.next(Promptable)` for example.
    Regular(Box<MethodCall<Promptable>>),
}

impl FieldPrompt {
    /// Returns the prompt call of the field from its prompt attribute and the message of the prompt.
    ///
    /// The message retrieval depends on the field type (named/unnamed).
    fn new(mut attr: Sp<RawFieldAttr>, field: Field, msg: String, gens: &mut Generics) -> Self {
        let kind = match (attr.val.optional, attr.val.or_default) {
            (true, true) => unreachable!("assert !(opt && or_default)"),
            (true, false) => PromptKind::NextOptional,
            (false, true) if is_ty(&field.ty, "Option") => abort_opt_or_default(attr.span),
            (false, true) => PromptKind::NextOrDefault,
            (false, false) if is_ty(&field.ty, "Option") => PromptKind::NextOptional,
            (false, false) => PromptKind::Next,
        };

        map_method_call!(attr: example "example", or_val "default_value", or_env "default_env", fmt "format");

        let w = Written {
            msg,
            fmt,
            example,
            or_val,
            or_env,
        };

        if attr.val.flatten {
            // Flattened prompt, we call `Prompted::from_values` method for this field,
            // so it needs to implement the Prompted trait.
            if let Some(id) = get_ty_ident(&field.ty) {
                let root = get_lib_root().1;
                // We check if the output value is a generic type.
                check_for_bound(gens, id, quote!(#root::menu::Prompted));
            }
            Self::Flatten(field.ty.span())
        } else {
            let prompt = if let Some(mut til) = attr.val.until.take() {
                if let FunctionExpr::Closure(expr) = &mut til {
                    // This is placed to avoid the user to provide the type
                    // of the argument in the closure.
                    insert_type_for(expr, &field.ty);
                }

                let inner = Box::new(Promptable::from_not_until(&field.ty, w, attr.val, gens));
                Promptable::Until(Until { inner, til })
            } else {
                Promptable::from_not_until(&field.ty, w, attr.val, gens)
            };

            // We call the `next` method with the promptable as argument.
            Self::Regular(kind.call_for(&field.ty, prompt))
        }
    }
}

impl ToTokens for FieldPrompt {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldPrompt::Flatten(sp) => {
                // We span the call expansion because the output type may not implement
                // the `Prompted` trait, and rustc will span the entire function call path.
                let root = get_lib_root_spanned(*sp).1;
                quote_spanned!(*sp=> #root::menu::Prompted::from_values(__vals)?).to_tokens(tokens);
            }
            FieldPrompt::Regular(call) => quote!(__vals #call).to_tokens(tokens),
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
fn get_field_prompt(field: Field, case: Option<&Case>, gens: &mut Generics) -> FieldPrompt {
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
    fn new(field: Field, case: Option<&Case>, gens: &mut Generics) -> Self {
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
        gens: &mut Generics,
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
    fn new(field: Field, case: Option<&Case>, gens: &mut Generics) -> Self {
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
    fn new(fields: Punctuated<Field, Token![,]>, case: Option<&Case>, gens: &mut Generics) -> Self {
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
fn build_unit_struct(attrs: Vec<Attribute>, name: Ident, gens: Generics) -> TokenStream {
    let low_name = map_unit_ident(&attrs, &name);
    let err_msg = format!("failed to parse to {name} struct");

    let (impl_gens, ty_gens, where_clause) = gens.split_for_impl();

    let root = get_lib_root().1;

    wrap_in_const(quote! {
        #[automatically_derived]
        impl #impl_gens #root::__private::FromStr for #name #ty_gens #where_clause {
            type Err = #root::__private::String;
            fn from_str(s: &#root::__private::str) -> #root::__private::Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    #low_name => #root::__private::Result::Ok(Self),
                    _ => #root::__private::Result::Err(#err_msg.to_owned()),
                }
            }
        }
    })
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

    quote!(writeln!(__vals.handle, #name)?;)
}

/// Returns the TokenStream of the struct construction.
///
/// This function is called after checking that the struct isn't an unit struct.
fn construct_ts(case: Option<&Case>, fields: Fields, gens: &mut Generics) -> TokenStream {
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
    used: TokenStream,
    attrs: Vec<Attribute>,
    name: Ident,
    mut gens: Generics,
    fields: Fields,
) -> TokenStream {
    // The name of the library.
    let root = get_lib_root().1;

    {
        let (impl_gens, ty_gens, where_clause) = gens.split_for_impl();
        set_dummy(wrap_in_const(quote! {
            #[automatically_derived]
            impl #impl_gens #root::menu::Prompted for #name #ty_gens #where_clause {
                fn from_values<__H: #root::menu::Handle>(_: &mut #root::menu::Values<__H>) -> #root::MenuResult<Self> {
                    #used
                    unimplemented!()
                }
            }
        }));
    }

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

    let (impl_gens, ty_gens, where_clause) = gens.split_for_impl();

    wrap_in_const(quote! {
        #[automatically_derived]
        impl #impl_gens #root::menu::Prompted for #name #ty_gens #where_clause {
            fn try_prompt_with<__H: #root::menu::Handle>(__handle: __H) -> #root::MenuResult<Self> {
                Self::from_values(&mut #root::menu::Values::from_handle(__handle) #fmt_fn)
            }

            fn from_values<__H: #root::menu::Handle>(__vals: &mut #root::menu::Values<__H>) -> #root::MenuResult<Self> {
                #used
                #disp_title
                #root::MenuResult::Ok(#init)
            }
        }
    })
}

/// Expands the `derive(Prompted)` macro for a struct.
fn build_struct(
    used: TokenStream,
    attrs: Vec<Attribute>,
    name: Ident,
    gens: Generics,
    fields: Fields,
) -> TokenStream {
    match fields {
        Fields::Unit => build_unit_struct(attrs, name, gens),
        other => build_fields_struct(used, attrs, name, gens, other),
    }
}

/// Expands the `derive(Prompted)` macro.
pub(crate) fn build_prompted(input: DeriveInput) -> TokenStream {
    let used = pretend_used(&input);

    match input.data {
        Data::Enum(DataEnum { variants, .. }) => {
            build_select(used, input.attrs, input.ident, input.generics, variants)
        }
        Data::Struct(DataStruct { fields, .. }) => {
            build_struct(used, input.attrs, input.ident, input.generics, fields)
        }
        _ => abort_call_site!("derive(Prompted) only supports enums and structs"),
    }
}

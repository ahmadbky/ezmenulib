use std::{fmt::Display, marker::PhantomData};

use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    AngleBracketedGenericArguments, Attribute, Expr, GenericArgument, Ident, Lit, Meta,
    MetaNameValue, Path, PathArguments, PathSegment, Token, Type, TypePath,
};

pub(crate) fn abort_duplicate_parameter(span: Span, s: &str) -> ! {
    abort!(span, "duplicate parameter: `{}`", s);
}

pub(crate) fn abort_conflict_param(span: Span, field: &str, with: &str) -> ! {
    abort!(
        span,
        "cannot provide `{}` parameter because `{}` is in conflict with it",
        field,
        with
    );
}

/// Internal macro used convert an object to a string slice
macro_rules! to_str {
    ($id:expr) => {
        $id.to_string().as_str()
    };
}

pub(crate) use to_str;

/// Wraps the code given in the argument into a const expression:
///
/// ```
/// const _: () = {
///     extern crate name_of_crate as _name_of_crate;
///     /* code */
/// };
/// ```
///
/// The reexport is used to avoid paths conflicts and hygiene.
pub(crate) fn wrap_in_const(code: TokenStream) -> TokenStream {
    let (name, reexport) = get_lib_root();

    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate #name as #reexport;
            #code
        };
    }
}

/// Returns the name of the library, and its reexport, to a call-site span,
/// because I am still not sure of the name of the library xd
#[inline(always)]
pub(crate) fn get_lib_root() -> (Ident, Ident) {
    get_lib_root_spanned(Span::call_site())
}

/// Returns the name of the library, and its reexport, to the given span,
/// because I am still not sure of the name of the library xd
#[inline(always)]
pub(crate) fn get_lib_root_spanned(span: Span) -> (Ident, Ident) {
    let name = Ident::new("ezmenulib", span);
    let reexport = format_ident!("_{}", name);
    (name, reexport)
}

/// Represents a spanned type.
///
/// This struct is used to save the span of an input code element, and to bind a type to it,
/// for error handling.
pub(crate) struct Sp<T> {
    pub(crate) span: Span,
    pub(crate) val: T,
}

impl<T: Default> Default for Sp<T> {
    fn default() -> Self {
        Self {
            span: Span::call_site(),
            val: Default::default(),
        }
    }
}

/// Convenient function used to only take the value of a spanned item.
///
/// # Basic usage
///
/// ```
/// let a: Option<Sp<i32>> = Some(Sp::default());
/// let a: Option<i32> = a.map(take_val);
/// ```
pub(crate) fn take_val<T>(sp: Sp<T>) -> T {
    sp.val
}

impl<T> Sp<T> {
    pub(crate) fn new(span: Span, val: T) -> Self {
        Self { span, val }
    }
}

impl<T: ToTokens> ToTokens for Sp<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.val.to_tokens(tokens);
    }
}

/// Util function used to return the attribute marked with the given identifier among
/// the given attributes.
pub(crate) fn get_attr<'a>(attrs: &'a [Attribute], ident: &str) -> Option<&'a Attribute> {
    attrs.iter().find(|attr| attr.path.is_ident(ident))
}

/// Util function used to parse the arguments of the attribute marked with the given identifier,
/// among the given attributes, to the output type.
pub(crate) fn get_attr_with_args<A: Parse>(attrs: &[Attribute], ident: &str) -> Option<Sp<A>> {
    get_attr(attrs, ident).map(|attr| {
        let val = attr
            .parse_args()
            .unwrap_or_else(|e| abort!(e.span(), "invalid attribute: {}", e));
        Sp::new(attr.span(), val)
    })
}

/// Util function used to get the first documentation line among the given attributes
/// of the concerned object.
pub(crate) fn get_first_doc(attrs: &[Attribute]) -> Option<String> {
    get_attr(attrs, "doc").and_then(|attr| match attr.parse_meta() {
        Ok(Meta::NameValue(MetaNameValue {
            lit: Lit::Str(lit), ..
        })) => Some(lit.value().trim_start_matches(' ').to_owned()),
        _ => None,
    })
}

/// Represents a method call.
///
/// A method call is expanded to `.name::<T0, T1, ...>(arg0, arg1, ...)[?]`
///
/// The T generic type parameter is only used for convenience purpose. For a method call that
/// doesn't take any argument, T is replaced by the unit type `()`. For a single parameter,
/// T is replaced by the parameter type. Otherwise, T is replaced by a tuple with
/// the types of the parameters.
pub(crate) struct MethodCall<T> {
    name: Ident,
    gens: Punctuated<Type, Token![,]>,
    args: Punctuated<Expr, Token![,]>,
    q: Option<Token![?]>,
    _marker: PhantomData<T>,
}

impl<T> MethodCall<T> {
    pub(crate) fn new(name: Ident, args: Punctuated<Expr, Token![,]>) -> Self {
        Self {
            name,
            gens: Punctuated::new(),
            args,
            q: None,
            _marker: PhantomData,
        }
    }

    pub(crate) fn with_span(mut self, span: Span) -> Self {
        self.name.set_span(span);
        self
    }

    pub(crate) fn with_generics(self, gens: Vec<Type>) -> Self {
        Self {
            gens: gens.into_iter().collect(),
            ..self
        }
    }

    pub(crate) fn with_question(self) -> Self {
        Self {
            q: Some(Token![?](Span::call_site())),
            ..self
        }
    }
}

/// Convenient function used to return a method call with a single parameter.
/// This is used to avoid the redundant `MethodCall::new(...)` call, because there are many
/// methods that takes only one parameter across the library.
pub(crate) fn method_call<T: ToTokens>(name: &str, arg: T) -> MethodCall<T> {
    MethodCall::new(Ident::new(name, Span::call_site()), parse_quote!(#arg))
}

/// Convenient function used to return a method call without any parameter.
/// Same as [`method_call`], this is used to avoid redundance.
pub(crate) fn method_call_empty(name: &str) -> MethodCall<()> {
    MethodCall::new(Ident::new(name, Span::call_site()), Punctuated::new())
}

impl<T> ToTokens for MethodCall<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let gens = &self.gens;
        let args = &self.args;
        quote!(.#name::<#gens>(#args)).to_tokens(tokens);
        self.q.to_tokens(tokens);
    }
}

/// Represents the type of case used to transform
#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum Case {
    /// The identifier is changed to uppercase.
    Upper,
    /// The identifier is changed to lowercase.
    Lower,
    /// The identifier isn't changed.
    #[default]
    Inherited,
}

impl Case {
    /// Method used to map a given string to its representation according to the case.
    pub(crate) fn map(&self, s: String) -> String {
        match self {
            Case::Upper => s.to_uppercase(),
            Case::Lower => s.to_lowercase(),
            Case::Inherited => s,
        }
    }
}

impl Parse for Case {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let id = input.parse::<Ident>()?;
        Ok(match to_str!(id) {
            "upper" => Self::Upper,
            "lower" => Self::Lower,
            "inherit" => Self::Inherited,
            other => abort!(
                id,
                "expected one of `upper`, `lower`, `inherit`, got `{}`",
                other
            ),
        })
    }
}

/// Replaces a char of the given string at the given index with the new piece.
fn replace_char<S: Display>(idx: usize, new: S, buf: &mut String) {
    buf.replace_range(idx..idx + 1, format!("{new}").as_str());
}

/// Returns the "prettified" version of a snake case ident.
///
/// "hello_everyone" => "Hello everyone"
pub(crate) fn split_ident_snake_case(id: &Ident) -> String {
    let mut out = id.to_string();
    let mut prev_up = false;
    let mut i = 0;

    while i < out.len() {
        let mut chars = out.chars().skip(i);
        // SAFETY: we just checked that i < out.len()
        let c = unsafe { chars.next().unwrap_unchecked() };

        if c == '_' {
            replace_char(i, ' ', &mut out);
        } else if c.is_uppercase() {
            match chars.next() {
                Some(next) if next.is_lowercase() && !prev_up && i > 0 => {
                    replace_char(i, c.to_lowercase(), &mut out);
                }
                _ => (),
            }
            prev_up = true;
        } else {
            if i == 0 {
                replace_char(i, c.to_uppercase(), &mut out);
            }
            prev_up = false;
        }

        i += 1;
    }

    out
}

/// Util function used to get the splitted version of an identifier written in the camel case.
///
/// It turns to lowercase the "tail" of the words inside.
///
/// "HelloEveryone" => "Hello everyone"
pub(crate) fn split_ident_camel_case(id: &Ident) -> String {
    let mut out = id.to_string();
    let mut prev_up = false;
    let mut i = 0;

    while i < out.len() {
        let mut chars = out.chars().skip(i);
        // SAFETY: we just checked that i < out.len()
        let c = unsafe { chars.next().unwrap_unchecked() };

        if c.is_uppercase() {
            if !prev_up && i > 0 && chars.next().filter(|c| c.is_lowercase()).is_some() {
                replace_char(i, c.to_lowercase(), &mut out);
                out.insert(i, ' ');
                i += 1;
            }

            prev_up = false;
        } else {
            prev_up = false;
        }

        i += 1;
    }

    out
}

/// Returns the last path segment of the given path.
pub(crate) fn get_last_seg_of_path(path: &Path) -> Option<&PathSegment> {
    path.segments.last()
}

/// Returns the nested generic arguments of a path segment.
pub(crate) fn get_nested_args(
    seg: &PathSegment,
) -> Option<&Punctuated<GenericArgument, Token![,]>> {
    match &seg.arguments {
        PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => Some(args),
        _ => None,
    }
}

/// Returns true if the path correponds to the given name, by checking its last segment,
/// otherwise returns false.
pub(crate) fn is_path(path: &Path, name: &str) -> bool {
    get_last_seg_of_path(path)
        .filter(|seg| seg.ident == name)
        .is_some()
}

/// Returns the last path segment of a type.
pub(crate) fn get_last_seg_of_ty(ty: &Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(TypePath { qself: None, path }) => Some(get_last_seg_of_path(path)?),
        _ => None,
    }
}

/// Returns the identifier of a given type.
pub(crate) fn get_ty_ident(ty: &Type) -> Option<&Ident> {
    if let Type::Path(TypePath { path, .. }) = ty {
        path.get_ident()
    } else {
        None
    }
}

/// Returns true if the type correponds to the given name, otherwise returns false.
pub(crate) fn is_ty(ty: &Type, name: &str) -> bool {
    get_last_seg_of_ty(ty)
        .filter(|seg| seg.ident == name)
        .is_some()
}

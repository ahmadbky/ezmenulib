use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, quote_spanned, ToTokens};
use spelling_corrector::corrector::SimpleCorrector;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Ident, Lit, Meta, MetaNameValue, Path, PathSegment, Token, Type, TypePath,
};

/// Internal macro used convert an object to a string slice
macro_rules! to_str {
    ($id:expr) => {
        $id.to_string().as_str()
    };
}

pub(crate) use {define_attr, to_str};

/// Util function used to return the token stream of the path of the library name.
///
/// This function exists because the library name might change in the future.
pub(crate) fn get_lib_root() -> TokenStream {
    quote!(::ezmenulib)
}

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
        let tok = self.val.to_token_stream();
        quote_spanned!(self.span=> #tok).to_tokens(tokens)
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

pub(crate) struct MethodCall<T> {
    name: Ident,
    gens: Punctuated<Type, Token![,]>,
    arg: T,
    q: Option<Token![?]>,
}

impl<T> MethodCall<T> {
    pub(crate) fn new(name: Ident, arg: T) -> Self {
        Self {
            name,
            gens: Punctuated::new(),
            arg,
            q: None,
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

pub(crate) fn method_call<T>(name: &str, arg: T) -> MethodCall<T> {
    let name = Ident::new(name, Span::call_site());
    MethodCall::new(name, arg)
}

impl<T: ToTokens> ToTokens for MethodCall<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let gens = &self.gens;
        let arg = &self.arg;
        let q = &self.q;
        quote!(.#name::<#gens>(#arg) #q).to_tokens(tokens);
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

        let ups = ["upper", "upper_case", "uppercase", "up"];
        let lows = ["lower", "lower_case", "lowercase", "low"];
        let inhs = ["inherit", "inherited", "inh"];

        match &to_str!(id) {
            s if ups.contains(s) => Ok(Self::Upper),
            s if lows.contains(s) => Ok(Self::Lower),
            s if inhs.contains(s) => Ok(Self::Inherited),
            _ => abort_invalid_ident(id, &[ups.as_ref(), lows.as_ref(), inhs.as_ref()].concat()),
        }
    }
}

fn replace_char<S: Display>(idx: usize, new: S, buf: &mut String) {
    buf.replace_range(idx..idx + 1, format!("{new}").as_str());
}

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
pub(crate) fn split_ident_camel_case(id: &Ident) -> String {
    let mut out = id.to_string();
    let mut prev_up = false;
    let mut i = 0;

    while i < out.len() {
        let mut chars = out.chars().skip(i);
        // SAFETY: we just checked that i < out.len()
        let c = unsafe { chars.next().unwrap_unchecked() };

        if c.is_uppercase() {
            if !prev_up && i > 0 {
                match chars.next() {
                    Some(next) if next.is_lowercase() => {
                        replace_char(i, c.to_lowercase(), &mut out);
                    }
                    _ => (),
                }
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

/// Returns the pretty version of the given array of string slices.
///
/// This surrounds each argument of the array with `...`,
/// and joins it with commas.
fn prettify(args: &[&str]) -> String {
    /// The maximum number of lines displayed
    const MAX: usize = 5;

    let mut lines: Vec<_> = args
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{} - `{s}`\n", i + 1))
        .take(MAX)
        .collect();
    if args.len() > MAX {
        lines.push("... and more".to_owned());
    }
    lines.join("")
}

/// Util function used to abort when an invalid identifier has been provided.
pub(crate) fn abort_invalid_ident(id: Ident, valids: &[&str]) -> ! {
    let corrector = SimpleCorrector::from_iter(valids.iter().copied());
    let opt_help = corrector
        .correct(to_str!(id))
        .map(|w| format!("did you mean `{w}`?"));
    abort!(
        id,
        "unexpected identifier: `{id}`. expected one of:\n{}", prettify(valids);
        help =? opt_help;
    );
}

pub fn get_last_seg_of(ty: &Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(TypePath {
            qself: None,
            path:
                Path {
                    leading_colon: None,
                    segments,
                },
        }) => Some(segments.last()?),
        _ => None,
    }
}

pub fn is_ty(ty: &Type, name: &str) -> bool {
    get_last_seg_of(ty)
        .filter(|seg| seg.ident == name)
        .is_some()
}

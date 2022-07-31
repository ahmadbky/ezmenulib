use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use spelling_corrector::corrector::SimpleCorrector;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Ident, Lit, Meta, MetaNameValue,
};

/// Internal macro used convert an object to a string slice
macro_rules! to_str {
    ($id:expr) => {
        $id.to_string().as_str()
    };
}

pub(crate) use to_str;

/// Util function used to return the token stream of the path of the library name.
///
/// This function exists because the library name might change in the future.
pub fn get_lib_root() -> TokenStream {
    quote!(::ezmenulib)
}

/// Util function used to return the attribute marked with the given identifier among
/// the given attributes.
pub fn get_attr<'a>(attrs: &'a [Attribute], ident: &str) -> Option<&'a Attribute> {
    attrs
        .iter()
        .find(|attr| attr.path.segments.iter().any(|seg| seg.ident == ident))
}

/// Util function used to parse the arguments of the attribute marked with the given identifier,
/// among the given attributes, to the output type.
pub fn get_attr_with_args<A: Parse>(attrs: &[Attribute], ident: &str) -> Option<A> {
    get_attr(attrs, ident).map(|attr| {
        attr.parse_args()
            .unwrap_or_else(|e| abort!(e.span(), "invalid attribute: {}", e))
    })
}

/// Util function used to get the first documentation line among the given attributes
/// of the concerned object.
pub fn get_first_doc(attrs: &[Attribute]) -> Option<String> {
    get_attr(attrs, "doc").and_then(|attr| match attr.parse_meta() {
        Ok(Meta::NameValue(MetaNameValue {
            lit: Lit::Str(lit), ..
        })) => Some(lit.value().trim_start_matches(' ').to_owned()),
        _ => None,
    })
}

/// Represents the type of case used to transform
#[derive(Debug, Clone, Copy, Default)]
pub enum Case {
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
    pub fn map(&self, s: String) -> String {
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

/// Util function used to get the splitted version of an identifier written in the camel case.
///
/// It turns to lowercase the "tail" of the words inside.
pub fn split_ident_camel_case(id: &Ident) -> String {
    let mut out = id.to_string();
    let mut prev_up = false;
    let mut i = 0;

    while i < out.len() {
        let mut chars = out.chars();
        let c = chars.nth(i).unwrap();

        if c.is_uppercase() {
            if !prev_up && i > 0 {
                match chars.next() {
                    Some(next) if next.is_lowercase() => {
                        out.replace_range(i..i + 1, c.to_lowercase().to_string().as_str());
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
pub fn abort_invalid_ident(id: Ident, valids: &[&str]) -> ! {
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

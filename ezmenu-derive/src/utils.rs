use proc_macro_error::abort;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Lit, LitBool, LitStr, Meta, MetaList, MetaNameValue, NestedMeta, Path, Token,
};

pub fn parse<T, MatchNested, MatchNameValue>(
    desc: &mut T,
    func_nested: MatchNested,
    func_nv: MatchNameValue,
    input: Meta,
) where
    MatchNested: Fn(&mut T, String, &NestedMeta),
    MatchNameValue: Fn(&mut T, String, Lit),
{
    match input {
        Meta::List(MetaList { nested, .. }) => {
            for nm in nested {
                match nm {
                    // #[menu(arg("..."), ...)]
                    // in inner metas, if the meta type is a list,
                    // then it should contain only 1 nested meta as value
                    // like a path to a function, or a string literal for a message
                    NestedMeta::Meta(Meta::List(MetaList { path, nested, .. })) => {
                        let nested = get_first_nested(&nested);
                        func_nested(desc, path_to_string(&path), nested);
                    }
                    // #[menu(arg = "...", ...)]
                    // deconstructing to a path and a literal
                    NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) => {
                        func_nv(desc, path_to_string(&path), lit)
                    }
                    _ => abort!(nm, "expected value definition"),
                }
            }
        }
        _ => abort_incorrect_def(&input),
    }
}

// get the first nested meta inside parenthesis
pub fn get_first_nested(nested: &Punctuated<NestedMeta, Token![,]>) -> &NestedMeta {
    let nested = nested.first();
    match nested {
        Some(nm) => nm,
        _ => abort!(nested, "value definition missing"),
    }
}

pub fn run_nv_bool(arg: &str, lit: Lit, val: &mut Option<LitBool>) {
    if let Lit::Bool(lit) = lit {
        *val = Some(lit);
    } else {
        abort_invalid_type(lit, arg);
    }
}

pub fn run_nv_str(arg: &str, lit: Lit, val: &mut Option<LitStr>) {
    if let Lit::Str(lit) = lit {
        *val = Some(lit);
    } else {
        abort_invalid_type(lit, arg);
    }
}

pub fn run_nested(arg: &str, nested: &NestedMeta, val: &mut Option<Lit>) {
    if let NestedMeta::Lit(lit) = nested {
        *val = Some(lit.clone());
    } else {
        abort_invalid_type(nested, arg);
    }
}

pub fn run_nested_bool(arg: &str, nested: &NestedMeta, val: &mut Option<LitBool>) {
    if let NestedMeta::Lit(Lit::Bool(lit)) = nested {
        *val = Some(lit.clone());
    } else {
        abort_invalid_type(nested, arg);
    }
}

pub fn run_nested_path(arg: &str, nested: &NestedMeta, val: &mut Option<Path>) {
    if let NestedMeta::Meta(Meta::Path(path)) = nested {
        *val = Some(path.clone());
    } else {
        abort_invalid_type(nested, arg);
    }
}

pub fn run_nested_str(arg: &str, nested: &NestedMeta, val: &mut Option<LitStr>) {
    if let NestedMeta::Lit(Lit::Str(lit)) = nested {
        *val = Some(lit.clone());
    } else {
        abort_invalid_type(nested, arg);
    }
}

#[inline(never)]
pub fn abort_invalid_type(span: impl ToTokens, s: &str) -> ! {
    abort!(span, "invalid literal type for `{}` attribute", s)
}

#[inline(never)]
pub fn abort_invalid_arg_name(span: impl ToTokens, s: &str) -> ! {
    abort!(span, "invalid argument name: `{}`", s)
}

#[inline(never)]
pub fn abort_incorrect_def(span: &impl ToTokens) -> ! {
    abort!(span, "incorrect definition of field attribute")
}

#[inline]
pub fn path_to_string(from: &Path) -> String {
    // meta attribute parsing makes path always start with an ident
    from.get_ident().unwrap().to_string()
}

/// Returns the "menu" attribute parsed to a Meta.
pub fn get_meta_attr(attrs: Vec<Attribute>) -> Option<Meta> {
    attrs.into_iter().find_map(|attr| {
        attr.path
            .segments
            .first()
            .filter(|seg| seg.ident == "menu")
            .map(|_| {
                attr.parse_meta()
                    .unwrap_or_else(|_| abort_incorrect_def(&attr))
            })
    })
}

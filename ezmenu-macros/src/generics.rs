//! Module that contains util functions to insert trait bounds in generics.

use proc_macro2::TokenStream;
use syn::{
    parse2, punctuated::Punctuated, Generics, Ident, Token, TraitBound, TypeParam, TypeParamBound,
};

use crate::utils::{get_last_seg_of_path, is_path, to_str};

const ERR_MSG: &str = "invalid trait path";

/// Returns true if the bound type parameters don't contain the given trait path,
/// otherwise returns false.
fn path_not_in(trait_path: &TokenStream, bounds: &Punctuated<TypeParamBound, Token![+]>) -> bool {
    let path = parse2(trait_path.clone()).expect(ERR_MSG);
    let seg = get_last_seg_of_path(&path).expect(ERR_MSG);
    bounds.iter().all(|bound| {
        !matches!(
            bound,
            TypeParamBound::Trait(
                TraitBound { path, .. }
            ) if is_path(path, to_str!(seg.ident))
        )
    })
}

/// Checks if the bound type parameters of the generics don't contain the given trait path bound
/// to the given ident, and pushes the trait path to the bound type parameters if so.
pub(crate) fn check_for_bound(generics: &mut Generics, id: &Ident, trait_path: TokenStream) {
    if let Some(bounds) = generics.type_params_mut().find_map(|param| match param {
        TypeParam { ident, bounds, .. } if ident == id && path_not_in(&trait_path, bounds) => {
            Some(bounds)
        }
        _ => None,
    }) {
        bounds.push(parse2(trait_path).expect(ERR_MSG));
    }
}

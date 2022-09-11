//! Module that basically turns a function into a bound function to be called by a menu.
//!
//! A bound function means a function that takes a `<H> &mut H` first parameter for a raw menu,
//! or a `<B: Backend> &mut Terminal<B>` first parameter for a tui menu.
//!
//! So this module inserts this parameter if not already present,
//! according to its `tui` argument.

use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, set_dummy};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, punctuated::Punctuated, FnArg, GenericArgument, Generics, Ident, ItemFn, Pat,
    PatType, Path, PathSegment, Receiver, Signature, Token, TraitBound, Type, TypeParamBound,
    TypePath, TypeReference,
};

use crate::{
    kw::define_attr,
    utils::{get_last_seg_of_path, get_lib_root, get_nested_args, is_path},
};

/// Util function used to abort when the user tries to bind an invalid function,
/// because its signature contains the `kw` keyword.
fn abort_invalid_fn(span: Span, kw: &str) -> ! {
    abort!(span, "bound function cannot be {}", kw);
}

/// Returns the identifier of the first generic parameter that is bound to the
/// Handle trait for a raw menu or the Backend trait for a tui menu.
///
/// This is used to avoid redundance in the expansion, in case there is already a specified
/// generic argument.
fn get_h_gen(tui: bool, gens: &mut Generics) -> Option<&Ident> {
    let id = if tui { "Backend" } else { "Handle" };
    gens.type_params().find_map(|param| {
        if param.bounds.iter().any(|bound| {
            matches!(
                bound,
                TypeParamBound::Trait(TraitBound { path, ..}) if is_path(path, id)
            )
        }) {
            Some(&param.ident)
        } else {
            None
        }
    })
}

/// Returns the identifier of the first generics parameter that is bound to the
/// Handle trait for a raw menu or the Backend trait for a tui menu,
/// or pushes a new generic type parameter with the trait bound then return its ident.
fn get_or_push_h_gen(tui: bool, gens: &mut Generics) -> Ident {
    if let Some(id) = get_h_gen(tui, gens) {
        id.clone()
    } else {
        let root = get_lib_root().0;
        let (id, path): (_, Path) = if tui {
            (
                "__Backend",
                parse_quote!(::#root::__private::tui::backend::Backend),
            )
        } else {
            ("__Handle", parse_quote!(::#root::menu::Handle))
        };
        let id = Ident::new(id, Span::call_site());
        gens.params.push(parse_quote!(#id: #path));
        id
    }
}

/// Checks the signature of the function and aborts if it is invalid.
fn check_sig(sig: &Signature) {
    if let Some(uns) = sig.unsafety {
        abort_invalid_fn(uns.span, "unsafe");
    } else if let Some(asy) = sig.asyncness {
        abort_invalid_fn(asy.span, "async");
    } else if let Some(FnArg::Receiver(Receiver { self_token, .. })) = sig.inputs.first() {
        abort_invalid_fn(self_token.span, "associated");
    }
}

/// Returns true if the given path segment corresponds to the first argument of a function
/// bound for a tui menu, meaning that it is wrote as `Terminal<id>`, otherwise returns false.
fn is_tui_seg(id: &Ident, seg: &PathSegment) -> bool {
    get_nested_args(seg)
        .filter(|p| {
            p.len() == 1
                && matches!(
                    &p[0],
                    GenericArgument::Type(
                        Type::Path(TypePath { path, .. })
                    ) if path.is_ident(id)
                )
        })
        .is_some()
        && seg.ident == "Terminal"
}

/// Returns true if the given path segment corresponds to the first argument of a function
/// bound for a raw menu, meaning that it is wrote as `id`, otherwise returns false.
fn is_reg_seg(id: &Ident, seg: &PathSegment) -> bool {
    seg.ident == *id
}

/// Returns true if the arguments of the function needs an insert for the bound argument,
/// otherwise returns false.
fn needs_insert(tui: bool, id: &Ident, args: &Punctuated<FnArg, Token![,]>) -> bool {
    let cmp = if tui { is_tui_seg } else { is_reg_seg };

    match args.first() {
        Some(FnArg::Typed(PatType { pat, ty, .. })) => !matches!((&**pat, &**ty),
            (
                Pat::Ident(_),
                Type::Reference(TypeReference {
                    mutability: Some(_),
                    elem,
                    ..
                }),
            ) if matches!(
                &**elem,
                Type::Path(TypePath { path, .. })
                if get_last_seg_of_path(path).filter(|seg| cmp(id, seg)).is_some()
            )
        ),
        _ => true,
    }
}

/// Appends the necessary argument to the signature of the function.
fn append_handle(tui: bool, input: &mut Signature) {
    check_sig(input);

    let root = get_lib_root().0;
    let id = get_or_push_h_gen(tui, &mut input.generics);
    let id_ts = if tui {
        quote!(::#root::__private::tui::Terminal<#id>)
    } else {
        id.to_token_stream()
    };

    if needs_insert(tui, &id, &input.inputs) {
        input.inputs.insert(0, parse_quote!(__handle: &mut #id_ts));
    }
}

/// Entry point for the `bound` attribute macro.
pub(crate) fn build_bound(args: BoundArgs, mut input: ItemFn) -> TokenStream {
    set_dummy(input.to_token_stream());
    append_handle(args.tui, &mut input.sig);

    input.into_token_stream()
}

define_attr! {
    /// Defines the arguments of the [`bound`] attribute macro.
    pub(crate) BoundArgs {
        tui: bool,
    }
}

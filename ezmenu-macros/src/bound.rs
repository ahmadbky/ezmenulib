use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, set_dummy};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    FnArg, GenericArgument, Generics, Ident, ItemFn, Pat, PatType, Path, PathSegment, Receiver,
    Signature, Token, TraitBound, Type, TypeParamBound, TypePath, TypeReference,
};

use crate::{
    kw::define_attr,
    utils::{get_last_seg_of_path, get_lib_root, get_nested_args, is_path},
};

fn abort_invalid_fn(span: Span, kw: &str) -> ! {
    abort!(span, "bound function cannot be {}", kw);
}

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

fn get_or_push_h_gen(tui: bool, gens: &mut Generics) -> Ident {
    if let Some(id) = get_h_gen(tui, gens) {
        id.clone()
    } else {
        let (id, path): (_, Path) = if tui {
            ("__Backend", parse_quote!(::tui::backend::Backend))
        } else {
            let root = get_lib_root();
            ("__Handle", parse_quote!(#root::menu::Handle))
        };
        let id = Ident::new(id, Span::call_site());
        gens.params.push(parse_quote!(#id: #path));
        id
    }
}

fn check_sig(sig: &Signature) {
    if let Some(uns) = sig.unsafety {
        abort_invalid_fn(uns.span, "unsafe");
    } else if let Some(asy) = sig.asyncness {
        abort_invalid_fn(asy.span, "async");
    } else if let Some(FnArg::Receiver(Receiver { self_token, .. })) = sig.inputs.first() {
        abort_invalid_fn(self_token.span, "associated");
    }
}

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

fn is_reg_seg(id: &Ident, seg: &PathSegment) -> bool {
    seg.ident == *id
}

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

fn append_handle(tui: bool, input: &mut Signature) {
    check_sig(input);

    let id = get_or_push_h_gen(tui, &mut input.generics);
    let id_ts = if tui {
        quote!(::tui::Terminal<#id>)
    } else {
        id.to_token_stream()
    };

    if needs_insert(tui, &id, &input.inputs) {
        input.inputs.insert(0, parse_quote!(__handle: &mut #id_ts));
    }
}

pub(crate) fn build_bound(tui: bool, mut input: ItemFn) -> TokenStream {
    set_dummy(input.to_token_stream());
    append_handle(tui, &mut input.sig);

    input.into_token_stream()
}

define_attr! {
    BoundArgs {
        tui: bool,
    }
}

pub(crate) fn parse_bound_args(input: ParseStream) -> syn::Result<bool> {
    BoundArgs::parse(input).map(|args| args.tui)
}

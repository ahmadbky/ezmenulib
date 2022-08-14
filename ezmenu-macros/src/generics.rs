use proc_macro2::{Delimiter, Group, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse2, punctuated::Punctuated, ConstParam, GenericParam, Generics, Ident, Lifetime,
    LifetimeDef, Token, TraitBound, TraitBoundModifier, TypeParam, TypeParamBound,
};

enum GenIdent {
    Lifetime(Lifetime),
    Ident(Ident),
}

impl From<&GenericParam> for GenIdent {
    fn from(param: &GenericParam) -> Self {
        match param {
            GenericParam::Type(TypeParam { ident, .. })
            | GenericParam::Const(ConstParam { ident, .. }) => Self::Ident(ident.clone()),
            GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => {
                Self::Lifetime(lifetime.clone())
            }
        }
    }
}

impl ToTokens for GenIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            GenIdent::Lifetime(l) => l.to_tokens(tokens),
            GenIdent::Ident(i) => i.to_tokens(tokens),
        }
    }
}

pub(crate) struct AugmentedGenerics {
    input: Generics,
    idents: Punctuated<GenIdent, Token![,]>,
}

impl From<Generics> for AugmentedGenerics {
    fn from(input: Generics) -> Self {
        let idents = input.params.iter().map(GenIdent::from).collect();
        Self { input, idents }
    }
}

impl AugmentedGenerics {
    pub(crate) fn check_for_bound(&mut self, id: &Ident, trait_path: TokenStream) {
        if let Some(bounds) = self.input.params.iter_mut().find_map(|param| match param {
            GenericParam::Type(TypeParam { ident, bounds, .. }) if ident == id => Some(bounds),
            _ => None,
        }) {
            // We don't check if the bound is already present because rustc doesn't check it neither.
            bounds.push(TypeParamBound::Trait(TraitBound {
                paren_token: None,
                modifier: TraitBoundModifier::None,
                lifetimes: None,
                path: parse2(trait_path).expect("invalid trait path"),
            }))
        }
    }

    pub(crate) fn impl_for(
        &self,
        trait_path: TokenStream,
        name: &Ident,
        block: TokenStream,
    ) -> TokenStream {
        let gens = &self.input;
        let mut out = quote!(impl #gens #trait_path for #name);

        self.input.lt_token.to_tokens(&mut out);
        self.idents.to_tokens(&mut out);
        self.input.gt_token.to_tokens(&mut out);
        self.input.where_clause.to_tokens(&mut out);
        out.append(Group::new(Delimiter::Brace, block));

        out
    }
}

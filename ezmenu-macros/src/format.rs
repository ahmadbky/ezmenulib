use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    LitBool, LitStr,
};

use crate::{kw::define_attr, utils::get_lib_root};

macro_rules! impl_fmt {
    ($( $field:ident: $ty:ty ),*) => {
        define_attr! ( FormatInner {$( $field: $ty, )*} );

        #[derive(Clone, Debug)]
        pub(crate) struct Format {
            inner: FormatInner,
            some_omitted: bool,
        }

        impl Default for Format {
            fn default() -> Self {
                Self {
                    inner: Default::default(),
                    some_omitted: true,
                }
            }
        }

        impl Parse for Format {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let inner = input.parse::<FormatInner>()?;
                let some_omitted = $(inner.$field.is_none())||*;
                Ok(Self { inner, some_omitted })
            }
        }

        impl ToTokens for Format {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                let root = get_lib_root();
                $(let $field = self.inner.$field.as_ref().map(|v| quote!($field: #v,)));*;
                let base_struct = self.some_omitted.then(|| quote!(..Default::default()));
                quote! {
                    #root::field::Format {
                        $(#$field)*
                        #base_struct
                    }
                }
                .to_tokens(tokens);
            }
        }
    }
}

impl_fmt! {
    prefix: Option<LitStr>,
    left_sur: Option<LitStr>,
    right_sur: Option<LitStr>,
    chip: Option<LitStr>,
    show_default: Option<LitBool>,
    suffix: Option<LitStr>,
    line_brk: Option<LitBool>
}

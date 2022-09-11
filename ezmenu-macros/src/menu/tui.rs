use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, Expr, ExprLit, Lit, LitStr};

use crate::utils::{get_lib_root, method_call, MethodCall};

use super::RawBlockAttr;

#[derive(Default)]
pub(super) struct Block {
    styled_title: Option<MethodCall<Expr>>,
    title_alignment: Option<MethodCall<Expr>>,
    border_style: Option<MethodCall<Expr>>,
    style: Option<MethodCall<Expr>>,
    borders: Option<MethodCall<Expr>>,
    border_type: Option<MethodCall<Expr>>,
}

impl Block {
    pub(super) fn from_title(title: String) -> Self {
        let title = Expr::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Str(LitStr::new(title.as_str(), Span::call_site())),
        });
        let root = get_lib_root().1;

        Self {
            styled_title: Some(method_call("title", title)),
            title_alignment: Some(method_call(
                "title_alignment",
                parse_quote!(#root::__private::tui::layout::Alignment::Center),
            )),
            borders: Some(method_call(
                "borders",
                parse_quote!(#root::__private::tui::widgets::Borders::all()),
            )),
            ..Default::default()
        }
    }
}

macro_rules! map_method_call {
    ($attr:ident: $( $field:ident $msg:expr ),*) => {
        Self {$(
            $field: $attr.$field.map(|v| method_call($msg, v))
        ),*}
    };
}

impl From<RawBlockAttr> for Block {
    fn from(raw: RawBlockAttr) -> Self {
        map_method_call! {raw:
            styled_title "title",
            title_alignment "title_alignment",
            border_style "border_style",
            style "style",
            borders "borders",
            border_type "border_type"
        }
    }
}

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root = get_lib_root().1;
        quote!(#root::__private::tui::widgets::Block::default()).to_tokens(tokens);
        self.styled_title.to_tokens(tokens);
        self.title_alignment.to_tokens(tokens);
        self.border_style.to_tokens(tokens);
        self.style.to_tokens(tokens);
        self.borders.to_tokens(tokens);
        self.border_type.to_tokens(tokens);
    }
}

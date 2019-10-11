use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use std::iter::once;
use syn::parse::{Nothing, Parse, ParseStream};
use syn::{Attribute, Error, Field, Ident, Index, LitInt, LitStr, Result, Token};

pub struct Display {
    pub fmt: LitStr,
    pub args: TokenStream,
}

impl Parse for Display {
    fn parse(input: ParseStream) -> Result<Self> {
        let fmt: LitStr = input.parse()?;
        let args = input.call(parse_token_expr)?;
        let mut display = Display { fmt, args };
        display.expand_shorthand();
        Ok(display)
    }
}

fn parse_token_expr(input: ParseStream) -> Result<TokenStream> {
    let mut tokens = TokenStream::new();
    let mut last_is_comma = false;
    while !input.is_empty() {
        if last_is_comma && input.peek(Token![.]) {
            if input.peek2(Ident) {
                input.parse::<Token![.]>()?;
                last_is_comma = false;
                continue;
            }
            if input.peek2(LitInt) {
                input.parse::<Token![.]>()?;
                let int: Index = input.parse()?;
                let ident = format_ident!("_{}", int.index, span = int.span);
                tokens.extend(once(TokenTree::Ident(ident)));
                last_is_comma = false;
                continue;
            }
        }
        last_is_comma = input.peek(Token![,]);
        let token: TokenTree = input.parse()?;
        tokens.extend(once(token));
    }
    Ok(tokens)
}

impl ToTokens for Display {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fmt = &self.fmt;
        let args = &self.args;
        tokens.extend(quote! {
            write!(formatter, #fmt #args)
        });
    }
}

pub fn is_source(field: &Field) -> Result<bool> {
    for attr in &field.attrs {
        if attr.path.is_ident("source") {
            syn::parse2::<Nothing>(attr.tokens.clone())?;
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn display(attrs: &[Attribute]) -> Result<Option<Display>> {
    let mut display = None;

    for attr in attrs {
        if attr.path.is_ident("error") {
            if display.is_some() {
                return Err(Error::new_spanned(
                    attr,
                    "only one #[error(...)] attribute is allowed",
                ));
            }
            display = Some(attr.parse_args()?);
        }
    }

    Ok(display)
}

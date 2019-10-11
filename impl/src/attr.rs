use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use std::iter::once;
use syn::parse::{Nothing, Parse, ParseStream};
use syn::{
    braced, bracketed, parenthesized, token, Attribute, Error, Field, Ident, Index, LitInt, LitStr,
    Result, Token,
};

pub struct Display {
    pub fmt: LitStr,
    pub args: TokenStream,
    pub was_shorthand: bool,
}

impl Parse for Display {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut display = Display {
            fmt: input.parse()?,
            args: parse_token_expr(input, false)?,
            was_shorthand: false,
        };
        display.expand_shorthand();
        Ok(display)
    }
}

fn parse_token_expr(input: ParseStream, mut last_is_comma: bool) -> Result<TokenStream> {
    let mut tokens = TokenStream::new();
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
        let token: TokenTree = if input.peek(token::Paren) {
            let content;
            let delimiter = parenthesized!(content in input);
            let nested = parse_token_expr(&content, true)?;
            let mut group = Group::new(Delimiter::Parenthesis, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else if input.peek(token::Brace) {
            let content;
            let delimiter = braced!(content in input);
            let nested = parse_token_expr(&content, true)?;
            let mut group = Group::new(Delimiter::Brace, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else if input.peek(token::Bracket) {
            let content;
            let delimiter = bracketed!(content in input);
            let nested = parse_token_expr(&content, true)?;
            let mut group = Group::new(Delimiter::Bracket, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else {
            input.parse()?
        };
        tokens.extend(once(token));
    }
    Ok(tokens)
}

impl ToTokens for Display {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fmt = &self.fmt;
        let args = &self.args;
        if self.was_shorthand && fmt.value() == "{}" {
            let arg = args.clone().into_iter().nth(1).unwrap();
            tokens.extend(quote! {
                std::fmt::Display::fmt(#arg, formatter)
            });
        } else if self.was_shorthand && fmt.value() == "{:?}" {
            let arg = args.clone().into_iter().nth(1).unwrap();
            tokens.extend(quote! {
                std::fmt::Debug::fmt(#arg, formatter)
            });
        } else {
            tokens.extend(quote! {
                write!(formatter, #fmt #args)
            });
        }
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

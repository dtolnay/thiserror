use crate::ast::Field;
use crate::attr::{Display, Trait};
use crate::scan_expr::scan_expr;
use crate::unraw::IdentUnraw;
use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use std::collections::{BTreeSet as Set, HashMap as Map};
use syn::ext::IdentExt;
use syn::parse::discouraged::Speculative;
use syn::parse::{ParseStream, Parser};
use syn::{Expr, Ident, Index, LitStr, Member, Result, Token};

impl Display<'_> {
    // Transform `"error {var}"` to `"error {}", var`.
    pub fn expand_shorthand(&mut self, fields: &[Field]) {
        let raw_args = self.args.clone();
        let mut named_args = explicit_named_args.parse2(raw_args).unwrap().named;
        let mut member_index = Map::new();
        for (i, field) in fields.iter().enumerate() {
            member_index.insert(&field.member, i);
        }

        let span = self.fmt.span();
        let fmt = self.fmt.value();
        let mut read = fmt.as_str();
        let mut out = String::new();
        let mut args = self.args.clone();
        let mut has_bonus_display = false;
        let mut implied_bounds = Set::new();

        let mut has_trailing_comma = false;
        if let Some(TokenTree::Punct(punct)) = args.clone().into_iter().last() {
            if punct.as_char() == ',' {
                has_trailing_comma = true;
            }
        }

        self.requires_fmt_machinery = self.requires_fmt_machinery || fmt.contains('}');

        while let Some(brace) = read.find('{') {
            self.requires_fmt_machinery = true;
            out += &read[..brace + 1];
            read = &read[brace + 1..];
            if read.starts_with('{') {
                out.push('{');
                read = &read[1..];
                continue;
            }
            let next = match read.chars().next() {
                Some(next) => next,
                None => return,
            };
            let member = match next {
                '0'..='9' => {
                    let int = take_int(&mut read);
                    let member = match int.parse::<u32>() {
                        Ok(index) => Member::Unnamed(Index { index, span }),
                        Err(_) => return,
                    };
                    if !member_index.contains_key(&member) {
                        out += int;
                        continue;
                    }
                    member
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let ident = Ident::new(take_ident(&mut read), span);
                    Member::Named(ident)
                }
                _ => continue,
            };
            if let Some(&field) = member_index.get(&member) {
                let end_spec = match read.find('}') {
                    Some(end_spec) => end_spec,
                    None => return,
                };
                let bound = match read[..end_spec].chars().next_back() {
                    Some('?') => Trait::Debug,
                    Some('o') => Trait::Octal,
                    Some('x') => Trait::LowerHex,
                    Some('X') => Trait::UpperHex,
                    Some('p') => Trait::Pointer,
                    Some('b') => Trait::Binary,
                    Some('e') => Trait::LowerExp,
                    Some('E') => Trait::UpperExp,
                    Some(_) | None => Trait::Display,
                };
                implied_bounds.insert((field, bound));
            }
            let formatvar = IdentUnraw::new(match &member {
                Member::Unnamed(index) => format_ident!("_{}", index),
                Member::Named(ident) => ident.clone(),
            });
            out += &formatvar.to_string();
            if !named_args.insert(formatvar.clone()) {
                // Already specified in the format argument list.
                continue;
            }
            if !has_trailing_comma {
                args.extend(quote_spanned!(span=> ,));
            }
            let local = formatvar.to_local();
            args.extend(quote_spanned!(span=> #formatvar = #local));
            if read.starts_with('}') && member_index.contains_key(&member) {
                has_bonus_display = true;
                args.extend(quote_spanned!(span=> .as_display()));
            }
            has_trailing_comma = false;
        }

        out += read;
        self.fmt = LitStr::new(&out, self.fmt.span());
        self.args = args;
        self.has_bonus_display = has_bonus_display;
        self.implied_bounds = implied_bounds;
    }
}

struct FmtArguments {
    named: Set<IdentUnraw>,
    unnamed: bool,
}

#[allow(clippy::unnecessary_wraps)]
fn explicit_named_args(input: ParseStream) -> Result<FmtArguments> {
    let ahead = input.fork();
    if let Ok(set) = try_explicit_named_args(&ahead) {
        input.advance_to(&ahead);
        return Ok(set);
    }

    let ahead = input.fork();
    if let Ok(set) = fallback_explicit_named_args(&ahead) {
        input.advance_to(&ahead);
        return Ok(set);
    }

    input.parse::<TokenStream>().unwrap();
    Ok(FmtArguments {
        named: Set::new(),
        unnamed: false,
    })
}

fn try_explicit_named_args(input: ParseStream) -> Result<FmtArguments> {
    let mut syn_full = None;
    let mut args = FmtArguments {
        named: Set::new(),
        unnamed: false,
    };

    while !input.is_empty() {
        input.parse::<Token![,]>()?;
        if input.is_empty() {
            break;
        }
        if input.peek(Ident::peek_any) && input.peek2(Token![=]) && !input.peek2(Token![==]) {
            let ident: IdentUnraw = input.parse()?;
            input.parse::<Token![=]>()?;
            args.named.insert(ident);
        } else {
            args.unnamed = true;
        }
        if *syn_full.get_or_insert_with(is_syn_full) {
            let ahead = input.fork();
            if ahead.parse::<Expr>().is_ok() {
                input.advance_to(&ahead);
                continue;
            }
        }
        scan_expr(input)?;
    }

    Ok(args)
}

fn fallback_explicit_named_args(input: ParseStream) -> Result<FmtArguments> {
    let mut args = FmtArguments {
        named: Set::new(),
        unnamed: false,
    };

    while !input.is_empty() {
        if input.peek(Token![,])
            && input.peek2(Ident::peek_any)
            && input.peek3(Token![=])
            && !input.peek3(Token![==])
        {
            input.parse::<Token![,]>()?;
            let ident: IdentUnraw = input.parse()?;
            input.parse::<Token![=]>()?;
            args.named.insert(ident);
        }
    }

    Ok(args)
}

fn is_syn_full() -> bool {
    // Expr::Block contains syn::Block which contains Vec<syn::Stmt>. In the
    // current version of Syn, syn::Stmt is exhaustive and could only plausibly
    // represent `trait Trait {}` in Stmt::Item which contains syn::Item. Most
    // of the point of syn's non-"full" mode is to avoid compiling Item and the
    // entire expansive syntax tree it comprises. So the following expression
    // being parsed to Expr::Block is a reliable indication that "full" is
    // enabled.
    let test = quote!({
        trait Trait {}
    });
    match syn::parse2(test) {
        Ok(Expr::Verbatim(_)) | Err(_) => false,
        Ok(Expr::Block(_)) => true,
        Ok(_) => unreachable!(),
    }
}

fn take_int<'a>(read: &mut &'a str) -> &'a str {
    let mut int_len = 0;
    for ch in read.chars() {
        match ch {
            '0'..='9' => int_len += 1,
            _ => break,
        }
    }
    let (int, rest) = read.split_at(int_len);
    *read = rest;
    int
}

fn take_ident<'a>(read: &mut &'a str) -> &'a str {
    let mut ident_len = 0;
    for ch in read.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => ident_len += 1,
            _ => break,
        }
    }
    let (ident, rest) = read.split_at(ident_len);
    *read = rest;
    ident
}

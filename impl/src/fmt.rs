use crate::ast::Field;
use crate::attr::Display;
use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned};
use std::collections::HashSet as Set;
use syn::{Ident, Index, LitStr, Member};

impl Display<'_> {
    // Transform `"error {var}"` to `"error {}", var`.
    pub fn expand_shorthand(&mut self, fields: &[Field]) {
        if !self.args.is_empty() {
            return;
        }

        let span = self.fmt.span();
        let fmt = self.fmt.value();
        let mut read = fmt.as_str();
        let mut out = String::new();
        let mut args = TokenStream::new();
        let mut has_bonus_display = false;
        let fields: Set<Member> = fields.iter().map(|f| f.member.clone()).collect();

        while let Some(brace) = read.find('{') {
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
                    match int.parse::<u32>() {
                        Ok(index) => Member::Unnamed(Index { index, span }),
                        Err(_) => return,
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let ident = take_ident(&mut read);
                    Member::Named(Ident::new(&ident, span))
                }
                _ => return,
            };
            let ident = match &member {
                Member::Unnamed(index) => format_ident!("_{}", index),
                Member::Named(ident) => ident.clone(),
            };
            args.extend(quote_spanned!(span=> , #ident));
            if read.starts_with('}') && fields.contains(&member) {
                has_bonus_display = true;
                args.extend(quote_spanned!(span=> .as_display()));
            }
        }

        out += read;
        self.fmt = LitStr::new(&out, self.fmt.span());
        self.args = args;
        self.was_shorthand = true;
        self.has_bonus_display = has_bonus_display;
    }
}

fn take_int(read: &mut &str) -> String {
    let mut int = String::new();
    for (i, ch) in read.char_indices() {
        match ch {
            '0'..='9' => int.push(ch),
            _ => {
                *read = &read[i..];
                break;
            }
        }
    }
    int
}

fn take_ident(read: &mut &str) -> String {
    let mut ident = String::new();
    for (i, ch) in read.char_indices() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => ident.push(ch),
            _ => {
                *read = &read[i..];
                break;
            }
        }
    }
    ident
}

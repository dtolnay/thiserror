use crate::ast::{ContainerKind, Field};
use crate::attr::{Display, Trait};
use crate::private;
use crate::scan_expr::scan_expr;
use crate::unraw::{IdentUnraw, MemberUnraw};
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned, ToTokens as _};
use std::collections::{BTreeSet, HashMap};
use std::iter;
use syn::ext::IdentExt;
use syn::parse::discouraged::Speculative;
use syn::parse::{Error, ParseStream, Parser, Result};
use syn::{Expr, Ident, Index, LitStr, Token};

impl Display<'_> {
    pub fn expand_shorthand(&mut self, fields: &[Field], container: ContainerKind) -> Result<()> {
        let raw_args = self.args.clone();
        let FmtArguments {
            named: user_named_args,
            first_unnamed,
        } = explicit_named_args.parse2(raw_args).unwrap();

        let mut member_index = HashMap::new();
        let mut extra_positional_arguments_allowed = true;
        for (i, field) in fields.iter().enumerate() {
            member_index.insert(&field.member, i);
            extra_positional_arguments_allowed &= matches!(&field.member, MemberUnraw::Named(_));
        }

        let span = self.fmt.span();
        let fmt = self.fmt.value();
        let mut read = fmt.as_str();
        let mut out = String::new();
        let mut has_bonus_display = false;
        let mut infinite_recursive = false;
        let mut implied_bounds = BTreeSet::new();
        let mut bindings = Vec::new();
        let mut macro_named_args = BTreeSet::new();
        // Track which field names are used in the format string for selective binding
        let mut used_field_names = BTreeSet::new();

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
                None => return Ok(()),
            };
            let member = match next {
                '0'..='9' => {
                    let int = take_int(&mut read);
                    if !extra_positional_arguments_allowed {
                        if let Some(first_unnamed) = &first_unnamed {
                            let msg = format!("ambiguous reference to positional arguments by number in a {container}; change this to a named argument");
                            return Err(Error::new_spanned(first_unnamed, msg));
                        }
                    }
                    match int.parse::<u32>() {
                        Ok(index) => MemberUnraw::Unnamed(Index { index, span }),
                        Err(_) => return Ok(()),
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    if read.starts_with("r#") {
                        continue;
                    }
                    let repr = take_ident(&mut read);
                    if repr == "_" {
                        // Invalid. Let rustc produce the diagnostic.
                        out += repr;
                        continue;
                    }
                    let ident = IdentUnraw::new(Ident::new(repr, span));
                    if user_named_args.contains(&ident) {
                        // Refers to a named argument written by the user, not to field.
                        out += repr;
                        continue;
                    }
                    MemberUnraw::Named(ident)
                }
                _ => continue,
            };
            let end_spec = match read.find('}') {
                Some(end_spec) => end_spec,
                None => return Ok(()),
            };
            let mut bonus_display = false;
            let bound = match read[..end_spec].chars().next_back() {
                Some('?') => Trait::Debug,
                Some('o') => Trait::Octal,
                Some('x') => Trait::LowerHex,
                Some('X') => Trait::UpperHex,
                Some('p') => Trait::Pointer,
                Some('b') => Trait::Binary,
                Some('e') => Trait::LowerExp,
                Some('E') => Trait::UpperExp,
                Some(_) => Trait::Display,
                None => {
                    bonus_display = true;
                    has_bonus_display = true;
                    Trait::Display
                }
            };
            infinite_recursive |= member == *"self" && bound == Trait::Display;
            let field = match member_index.get(&member) {
                Some(&field) => field,
                None => {
                    out += &member.to_string();
                    continue;
                }
            };
            implied_bounds.insert((field, bound));
            let formatvar_prefix = if bonus_display {
                "__display"
            } else if bound == Trait::Pointer {
                "__pointer"
            } else {
                "__field"
            };
            let mut formatvar = IdentUnraw::new(match &member {
                MemberUnraw::Unnamed(index) => format_ident!("{}{}", formatvar_prefix, index),
                MemberUnraw::Named(ident) => {
                    format_ident!("{}_{}", formatvar_prefix, ident.to_string())
                }
            });
            while user_named_args.contains(&formatvar) {
                formatvar = IdentUnraw::new(format_ident!("_{}", formatvar.to_string()));
            }
            formatvar.set_span(span);
            out += &formatvar.to_string();
            if !macro_named_args.insert(formatvar.clone()) {
                // Already added to bindings by a previous use.
                continue;
            }
            let mut binding_value = match &member {
                MemberUnraw::Unnamed(index) => format_ident!("_{}", index),
                MemberUnraw::Named(ident) => ident.to_local(),
            };
            binding_value.set_span(span.resolved_at(fields[field].member.span()));
            // Track the original binding name for selective variable binding
            used_field_names.insert(binding_value.to_string());
            let wrapped_binding_value = if bonus_display {
                quote_spanned!(span=> #binding_value.as_display())
            } else if bound == Trait::Pointer {
                quote!(::thiserror::#private::Var(#binding_value))
            } else {
                binding_value.into_token_stream()
            };
            bindings.push((formatvar.to_local(), wrapped_binding_value));
        }

        out += read;
        self.fmt = LitStr::new(&out, self.fmt.span());
        self.has_bonus_display = has_bonus_display;
        self.infinite_recursive = infinite_recursive;
        self.implied_bounds = implied_bounds;
        self.bindings = bindings;
        self.used_field_names = used_field_names;

        // Also scan args for field references like .field or .0
        // These are used directly in expressions and need to be bound in the pattern
        self.used_field_names.extend(scan_field_refs(&self.args, fields));

        Ok(())
    }
}

struct FmtArguments {
    named: BTreeSet<IdentUnraw>,
    first_unnamed: Option<TokenStream>,
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
        named: BTreeSet::new(),
        first_unnamed: None,
    })
}

fn try_explicit_named_args(input: ParseStream) -> Result<FmtArguments> {
    let mut syn_full = None;
    let mut args = FmtArguments {
        named: BTreeSet::new(),
        first_unnamed: None,
    };

    while !input.is_empty() {
        input.parse::<Token![,]>()?;
        if input.is_empty() {
            break;
        }

        let mut begin_unnamed = None;
        if input.peek(Ident::peek_any) && input.peek2(Token![=]) && !input.peek2(Token![==]) {
            let ident: IdentUnraw = input.parse()?;
            input.parse::<Token![=]>()?;
            args.named.insert(ident);
        } else {
            begin_unnamed = Some(input.fork());
        }

        let ahead = input.fork();
        if *syn_full.get_or_insert_with(is_syn_full) && ahead.parse::<Expr>().is_ok() {
            input.advance_to(&ahead);
        } else {
            scan_expr(input)?;
        }

        if let Some(begin_unnamed) = begin_unnamed {
            if args.first_unnamed.is_none() {
                args.first_unnamed = Some(between(&begin_unnamed, input));
            }
        }
    }

    Ok(args)
}

fn fallback_explicit_named_args(input: ParseStream) -> Result<FmtArguments> {
    let mut args = FmtArguments {
        named: BTreeSet::new(),
        first_unnamed: None,
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
        } else {
            input.parse::<TokenTree>()?;
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

fn between<'a>(begin: ParseStream<'a>, end: ParseStream<'a>) -> TokenStream {
    let end = end.cursor();
    let mut cursor = begin.cursor();
    let mut tokens = TokenStream::new();

    while cursor < end {
        let (tt, next) = cursor.token_tree().unwrap();

        if end < next {
            if let Some((inside, _span, _after)) = cursor.group(Delimiter::None) {
                cursor = inside;
                continue;
            }
            if tokens.is_empty() {
                tokens.extend(iter::once(tt));
            }
            break;
        }

        tokens.extend(iter::once(tt));
        cursor = next;
    }

    tokens
}

/// Scans a TokenStream for field binding references that were already transformed.
/// The `parse_token_expr` function in `attr.rs` transforms `.field` to `field` and `.0` to `_0`.
/// So we look for these already-transformed identifiers.
/// Returns the set of binding names that need to be available in the pattern.
fn scan_field_refs(tokens: &TokenStream, fields: &[Field]) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    let mut tokens = tokens.clone().into_iter().peekable();

    // Build sets of valid binding names for validation
    // For named fields: binding name is the local binding name (via to_local())
    // For unnamed fields: binding name is "_0", "_1", etc.
    let valid_binding_names: BTreeSet<String> = fields
        .iter()
        .map(|f| match &f.member {
            MemberUnraw::Named(ident) => ident.to_local().to_string(),
            MemberUnraw::Unnamed(index) => format_ident!("_{}", index).to_string(),
        })
        .collect();

    while let Some(token) = tokens.next() {
        // Look for Ident tokens that match field binding names
        if let TokenTree::Ident(ident) = &token {
            let name = ident.to_string();
            // Check if this is a field binding name
            if valid_binding_names.contains(&name) {
                used.insert(name);
            }
        }

        // Recursively scan inside groups (parentheses, brackets, braces)
        if let TokenTree::Group(group) = &token {
            let inner_used = scan_field_refs(&group.stream(), fields);
            used.extend(inner_used);
        }
    }

    used
}

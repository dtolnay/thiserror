use crate::{ast::Field, attr::Display};
use proc_macro2::{Span, TokenTree};
use quote::{format_ident, quote_spanned};
use std::collections::HashSet as Set;
use syn::{
    ext::IdentExt,
    parse::{ParseStream, Parser},
    Ident, Index, LitStr, Member, Result, Token,
};

#[derive(Clone, Copy)]
pub enum DisplayFormatMarking<'a> {
    Debug(&'a crate::ast::Field<'a>),
    Display(&'a crate::ast::Field<'a>),
}

impl<'a> Display<'a> {
    // Transform `"error {var}"` to `"error {}", var`.
    pub fn expand_shorthand(&mut self, fields: &[Field]) {
        let raw_args = self.args.clone();
        let mut named_args = explicit_named_args.parse2(raw_args).unwrap();
        let fields: Set<Member> = fields.iter().map(|f| f.member.clone()).collect();

        let span = self.fmt.span();
        let fmt = self.fmt.value();
        let mut read = fmt.as_str();
        let mut out = String::new();
        let mut args = self.args.clone();
        let mut has_bonus_display = false;

        let mut has_trailing_comma = false;
        if let Some(TokenTree::Punct(punct)) = args.clone().into_iter().last() {
            if punct.as_char() == ',' {
                has_trailing_comma = true;
            }
        }

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
                    let member = match int.parse::<u32>() {
                        Ok(index) => Member::Unnamed(Index { index, span }),
                        Err(_) => return,
                    };
                    if !fields.contains(&member) {
                        out += &int;
                        continue;
                    }
                    member
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut ident = take_ident(&mut read);
                    ident.set_span(span);
                    Member::Named(ident)
                }
                _ => continue,
            };
            let local = match &member {
                Member::Unnamed(index) => format_ident!("_{}", index),
                Member::Named(ident) => ident.clone(),
            };
            let mut formatvar = local.clone();
            if formatvar.to_string().starts_with("r#") {
                formatvar = format_ident!("r_{}", formatvar);
            }
            if formatvar.to_string().starts_with('_') {
                // Work around leading underscore being rejected by 1.40 and
                // older compilers. https://github.com/rust-lang/rust/pull/66847
                formatvar = format_ident!("field_{}", formatvar);
            }
            out += &formatvar.to_string();
            if !named_args.insert(formatvar.clone()) {
                // Already specified in the format argument list.
                continue;
            }
            if !has_trailing_comma {
                args.extend(quote_spanned!(span=> ,));
            }
            args.extend(quote_spanned!(span=> #formatvar = #local));
            if read.starts_with('}') && fields.contains(&member) {
                has_bonus_display = true;
                args.extend(quote_spanned!(span=> .as_display()));
            }
            has_trailing_comma = false;
        }

        out += read;
        self.fmt = LitStr::new(&out, self.fmt.span());
        self.args = args;
        self.has_bonus_display = has_bonus_display;
    }

    pub fn iter_fmt_types(&'a self, fields: &'a [Field]) -> Vec<DisplayFormatMarking<'a>> {
        let members: Set<Member> = fields.iter().map(|f| f.member.clone()).collect();
        let fmt = self.fmt.value();
        let read = fmt.as_str();

        let mut member_refs: Vec<(&Member, &Field, bool)> = Vec::new();

        for template in parse_fmt_template(read) {
            if let Some(target) = &template.target {
                if members.contains(target) {
                    if let Some(matching) = fields.iter().find(|f| &f.member == target) {
                        member_refs.push((&matching.member, matching, template.is_display()));
                    }
                }
            }
        }

        member_refs
            .iter()
            .map(|(_m, f, is_display)| {
                if *is_display {
                    DisplayFormatMarking::Display(*f)
                } else {
                    DisplayFormatMarking::Debug(*f)
                }
            })
            .collect()
    }
}

struct FormatInterpolation {
    pub target: Option<Member>,
    pub format: Option<String>,
}

impl FormatInterpolation {
    pub fn is_debug(&self) -> bool {
        self.format
            .as_ref()
            .map(|x| x.contains('?'))
            .unwrap_or(false)
    }

    pub fn is_display(&self) -> bool {
        !self.is_debug()
    }
}

impl From<(Option<&str>, Option<&str>)> for FormatInterpolation {
    fn from((target, style): (Option<&str>, Option<&str>)) -> Self {
        let target = match target {
            None => None,
            Some(s) => Some(if let Ok(i) = s.parse::<usize>() {
                Member::Unnamed(syn::Index {
                    index: i as u32,
                    span: Span::call_site(),
                })
            } else {
                let mut s = s;
                let ident = take_ident(&mut s);
                Member::Named(ident)
            }),
        };
        let format = style.map(String::from);
        FormatInterpolation { target, format }
    }
}

fn read_format_template(mut read: &str) -> Option<(FormatInterpolation, &str)> {
    // If we aren't in a bracketed area, or we are in an escaped bracket, return None
    if !read.starts_with('{') || read.starts_with("{{") {
        return None;
    }
    // Read past the starting bracket
    read = &read[1..];
    // If there is no end bracket, bail
    let end_bracket = read.find('}')?;
    let contents = &read[..end_bracket];
    let (name, style) = if let Some(colon) = contents.find(':') {
        (&contents[..colon], &contents[colon + 1..])
    } else {
        (contents, "")
    };

    // Strip expanded identifier-prefixes since we just want the non-shorthand version
    let name = if name.starts_with("field_") {
        &name["field_".len()..]
    } else if name.starts_with("r_") {
        &name["r_".len()..]
    } else {
        name
    };
    let name = if name.starts_with('_') {
        &name["_".len()..]
    } else {
        name
    };

    let name = if name.is_empty() { None } else { Some(name) };
    let style = if style.is_empty() { None } else { Some(style) };
    Some(((name, style).into(), &read[end_bracket + 1..]))
}

fn parse_fmt_template(mut read: &str) -> Vec<FormatInterpolation> {
    let mut output = Vec::new();
    // From each "{", try reading a template; double-bracket escape handling is done by the template reader
    while let Some(opening_bracket) = read.find('{') {
        read = &read[opening_bracket..];
        if let Some((template, next)) = read_format_template(read) {
            read = next;
            output.push(template);
        }
        read = &read[read.char_indices().nth(1).map(|(x, _)| x).unwrap_or(0)..];
    }
    output
}

fn explicit_named_args(input: ParseStream) -> Result<Set<Ident>> {
    let mut named_args = Set::new();

    while !input.is_empty() {
        if input.peek(Token![,]) && input.peek2(Ident::peek_any) && input.peek3(Token![=]) {
            input.parse::<Token![,]>()?;
            let ident = input.call(Ident::parse_any)?;
            input.parse::<Token![=]>()?;
            named_args.insert(ident);
        } else {
            input.parse::<TokenTree>()?;
        }
    }

    Ok(named_args)
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

fn take_ident(read: &mut &str) -> Ident {
    let mut ident = String::new();
    let raw = read.starts_with("r#");
    if raw {
        ident.push_str("r#");
        *read = &read[2..];
    }
    for (i, ch) in read.char_indices() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => ident.push(ch),
            _ => {
                *read = &read[i..];
                break;
            }
        }
    }
    Ident::parse_any.parse_str(&ident).unwrap()
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use syn::Member;

    use super::{parse_fmt_template, FormatInterpolation};

    #[test]
    fn parse_and_emit_format_strings() {
        let test_str = "\"hello world {{{:} {x:#?} {1} {2:}\"";
        let template_groups = parse_fmt_template(test_str);
        assert!(matches!(
            &template_groups[0],
            FormatInterpolation {
                target: None,
                format: None
            }
        ));
        assert!(
            matches!(&template_groups[1], FormatInterpolation { target: Some(Member::Named(x)), format: Some(fmt) } if x.to_token_stream().to_string() == "x" && fmt == "#?")
        );
        assert!(
            matches!(&template_groups[2], FormatInterpolation { target: Some(Member::Unnamed(idx)), format: None } if idx.index == 1)
        );
        assert!(
            matches!(&template_groups[3], FormatInterpolation { target: Some(Member::Unnamed(idx)), format: None } if idx.index == 2)
        );
    }
}

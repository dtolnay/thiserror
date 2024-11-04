use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use std::cmp::Ordering;
use std::fmt::{self, Display};
use syn::ext::IdentExt as _;
use syn::parse::{Parse, ParseStream, Result};

#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct IdentUnraw(Ident);

impl IdentUnraw {
    pub fn new(ident: Ident) -> Self {
        IdentUnraw(ident)
    }

    pub fn to_local(&self) -> Ident {
        let unraw = self.0.unraw();
        let repr = unraw.to_string();
        if syn::parse_str::<Ident>(&repr).is_err() {
            if let "_" | "super" | "self" | "Self" | "crate" = repr.as_str() {
                // Some identifiers are never allowed to appear as raw, like r#self and r#_.
            } else {
                return Ident::new_raw(&repr, Span::call_site());
            }
        }
        unraw
    }
}

impl Display for IdentUnraw {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0.unraw(), formatter)
    }
}

impl Eq for IdentUnraw {}

impl PartialEq for IdentUnraw {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.0.unraw(), &other.0.unraw())
    }
}

impl Ord for IdentUnraw {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.0.unraw(), &other.0.unraw())
    }
}

impl PartialOrd for IdentUnraw {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Self::cmp(self, other))
    }
}

impl Parse for IdentUnraw {
    fn parse(input: ParseStream) -> Result<Self> {
        input.call(Ident::parse_any).map(IdentUnraw::new)
    }
}

impl ToTokens for IdentUnraw {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.unraw().to_tokens(tokens);
    }
}

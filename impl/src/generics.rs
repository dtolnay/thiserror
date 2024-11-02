use proc_macro2::TokenStream;
use quote::ToTokens;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap as Map, BTreeSet as Set};
use syn::punctuated::Punctuated;
use syn::{
    parse_quote, GenericArgument, Generics, Ident, PathArguments, Token, Type, WhereClause,
    WherePredicate,
};

pub struct ParamsInScope<'a> {
    names: Set<&'a Ident>,
}

impl<'a> ParamsInScope<'a> {
    pub fn new(generics: &'a Generics) -> Self {
        ParamsInScope {
            names: generics.type_params().map(|param| &param.ident).collect(),
        }
    }

    pub fn intersects(&self, ty: &Type) -> bool {
        let mut found = false;
        crawl(self, ty, &mut found);
        found
    }
}

fn crawl(in_scope: &ParamsInScope, ty: &Type, found: &mut bool) {
    if let Type::Path(ty) = ty {
        if ty.qself.is_none() {
            if let Some(ident) = ty.path.get_ident() {
                if in_scope.names.contains(ident) {
                    *found = true;
                }
            }
        }
        ty.path
            .segments
            .iter()
            .filter_map(|segment| match &segment.arguments {
                PathArguments::AngleBracketed(arguments) => Some(&arguments.args),
                _ => None,
            })
            .flatten()
            .filter_map(|arg| match arg {
                GenericArgument::Type(ty) => Some(ty),
                _ => None,
            })
            .for_each(|ty| crawl(in_scope, ty, found));
    }
}

#[derive(Default)]
pub struct InferredBounds {
    bounds: Map<String, (Set<String>, Punctuated<TokenStream, Token![+]>)>,
    order: Vec<TokenStream>,
}

impl InferredBounds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, ty: impl ToTokens, bound: impl ToTokens) {
        let ty = ty.to_token_stream();
        let bound = bound.to_token_stream();
        let entry = self.bounds.entry(ty.to_string());
        if let Entry::Vacant(_) = entry {
            self.order.push(ty);
        }
        let (set, tokens) = entry.or_default();
        if set.insert(bound.to_string()) {
            tokens.push(bound);
        }
    }

    pub fn augment_where_clause(&self, generics: &Generics) -> WhereClause {
        let mut generics = generics.clone();
        let where_clause = generics.make_where_clause();
        where_clause
            .predicates
            .extend(self.order.iter().map(|ty| -> WherePredicate {
                let (_set, bounds) = &self.bounds[&ty.to_string()];
                parse_quote!(#ty: #bounds)
            }));
        generics.where_clause.unwrap()
    }
}

impl<T: ToTokens, B: ToTokens> Extend<(T, B)> for InferredBounds {
    fn extend<I: IntoIterator<Item = (T, B)>>(&mut self, iter: I) {
        iter.into_iter()
            .for_each(|(ty, bound)| self.insert(ty, bound));
    }
}

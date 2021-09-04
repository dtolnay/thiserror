use std::collections::BTreeSet as Set;
use syn::{GenericArgument, Generics, Ident, PathArguments, Type};

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
        for segment in &ty.path.segments {
            if let PathArguments::AngleBracketed(arguments) = &segment.arguments {
                for arg in &arguments.args {
                    if let GenericArgument::Type(ty) = arg {
                        crawl(in_scope, ty, found);
                    }
                }
            }
        }
    }
}

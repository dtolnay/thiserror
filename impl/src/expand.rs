use crate::ast::{Enum, Field, Input, Struct};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_quote, Data, DeriveInput, Member, PathArguments, Result, Type, Visibility};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    let input = Input::from_syn(node)?;
    input.validate()?;
    Ok(match input {
        Input::Struct(input) => impl_struct(input),
        Input::Enum(input) => impl_enum(input),
    })
}

fn impl_struct(input: Struct) -> TokenStream {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let source_body = if input.attrs.transparent.is_some() {
        let only_field = &input.fields[0].member;
        Some(quote! {
            std::error::Error::source(self.#only_field.as_dyn_error())
        })
    } else if let Some(source_field) = input.source_field() {
        let source = &source_field.member;
        let asref = if type_is_option(source_field.ty) {
            Some(quote_spanned!(source.span()=> .as_ref()?))
        } else {
            None
        };
        let dyn_error = quote_spanned!(source.span()=> self.#source #asref.as_dyn_error());
        Some(quote! {
            std::option::Option::Some(#dyn_error)
        })
    } else {
        None
    };
    let source_method = source_body.map(|body| {
        quote! {
            fn source(&self) -> std::option::Option<&(dyn std::error::Error + 'static)> {
                use thiserror::private::AsDynError;
                #body
            }
        }
    });

    let backtrace_method = input.backtrace_field().map(|backtrace_field| {
        let backtrace = &backtrace_field.member;
        let body = if let Some(source_field) = input.source_field() {
            let source = &source_field.member;
            let source_backtrace = if type_is_option(source_field.ty) {
                quote_spanned! {source.span()=>
                    self.#source.as_ref().and_then(|source| source.as_dyn_error().backtrace())
                }
            } else {
                quote_spanned! {source.span()=>
                    self.#source.as_dyn_error().backtrace()
                }
            };
            let combinator = if type_is_option(backtrace_field.ty) {
                quote! {
                    #source_backtrace.or(self.#backtrace.as_ref())
                }
            } else {
                quote! {
                    std::option::Option::Some(#source_backtrace.unwrap_or(&self.#backtrace))
                }
            };
            quote! {
                use thiserror::private::AsDynError;
                #combinator
            }
        } else if type_is_option(backtrace_field.ty) {
            quote! {
                self.#backtrace.as_ref()
            }
        } else {
            quote! {
                std::option::Option::Some(&self.#backtrace)
            }
        };
        quote! {
            fn backtrace(&self) -> std::option::Option<&std::backtrace::Backtrace> {
                #body
            }
        }
    });

    let display_body = if input.attrs.transparent.is_some() {
        let only_field = &input.fields[0].member;
        Some(quote! {
            std::fmt::Display::fmt(&self.#only_field, __formatter)
        })
    } else if let Some(display) = &input.attrs.display {
        let use_as_display = if display.has_bonus_display {
            Some(quote! {
                #[allow(unused_imports)]
                use thiserror::private::{DisplayAsDisplay, PathAsDisplay};
            })
        } else {
            None
        };
        let pat = fields_pat(&input.fields);
        Some(quote! {
            #use_as_display
            #[allow(unused_variables, deprecated)]
            let Self #pat = self;
            #display
        })
    } else {
        None
    };
    let display_impl = display_body.map(|body| {
        let mut extra_predicates = Vec::new();
        for field in input
            .attrs
            .display
            .iter()
            .flat_map(|d| d.iter_fmt_types(input.fields.as_slice()))
        {
            use crate::attr::DisplayFormatMarking;
            let (ty, bound, ast): (
                &syn::Type,
                syn::punctuated::Punctuated<syn::TypeParamBound, _>,
                &syn::Field,
            );
            match field {
                DisplayFormatMarking::Debug(f) => {
                    ty = f.ty;
                    bound = parse_quote! { ::std::fmt::Debug };
                    ast = f.original;
                }
                DisplayFormatMarking::Display(f) => {
                    ty = f.ty;
                    bound = parse_quote! { ::std::fmt::Display };
                    ast = f.original;
                }
            }
            // If a generic is at all present, a constraint will be applied to the field type
            // This may create redundant `AlwaysDebug<T>: Debug` scenarios, but covers T: Debug and &T: Debug cleanly
            let mut usages = GenericUsageVisitor::new_unmarked(
                input.generics.type_params().map(|p| p.ident.clone()),
            );
            syn::visit::visit_field(&mut usages, ast);
            if usages.iter_marked().next().is_some() {
                extra_predicates.push(syn::WherePredicate::Type(syn::PredicateType {
                    bounded_ty: ty.clone(),
                    colon_token: syn::token::Colon::default(),
                    bounds: bound,
                    lifetimes: None,
                }));
            }
        }

        let where_clause = augment_where_clause(where_clause, extra_predicates);

        quote! {
            #[allow(unused_qualifications)]
            impl #impl_generics std::fmt::Display for #ty #ty_generics #where_clause {
                #[allow(
                    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
                    clippy::nonstandard_macro_braces,
                    clippy::used_underscore_binding,
                )]
                fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    #body
                }
            }
        }
    });

    let from_impl = input.from_field().map(|from_field| {
        let backtrace_field = input.backtrace_field();
        let from = from_field.ty;
        let body = from_initializer(from_field, backtrace_field);
        quote! {
            #[allow(unused_qualifications)]
            impl #impl_generics std::convert::From<#from> for #ty #ty_generics #where_clause {
                #[allow(deprecated)]
                fn from(source: #from) -> Self {
                    #ty #body
                }
            }
        }
    });

    let (generic_field_types, generics_in_from_types): (Vec<&syn::Type>, Vec<proc_macro2::Ident>) = {
        let mut generics_in_from_types = GenericUsageVisitor::new_unmarked(
            input.generics.type_params().map(|p| p.ident.clone()),
        );
        let mut generic_field_types = Vec::new();
        if let Some(from_field) = input.from_field() {
            let mut generics_in_this_field = GenericUsageVisitor::new_unmarked(
                input.generics.type_params().map(|p| p.ident.clone()),
            );
            syn::visit::visit_type(&mut generics_in_this_field, &from_field.original.ty);
            if generics_in_from_types.mark_from(generics_in_this_field.iter_marked()) {
                generic_field_types.push(from_field.ty);
            }
        }
        (
            generic_field_types,
            generics_in_from_types.generics.into_keys().collect(),
        )
    };

    let extra_predicates =
        std::iter::once(parse_quote! { Self: ::std::fmt::Display + ::std::fmt::Debug })
            .chain(
                generics_in_from_types
                    .into_iter()
                    .map(|generic| parse_quote! { #generic: 'static }),
            )
            .chain(
                generic_field_types
                    .into_iter()
                    .map(|ty| parse_quote! { #ty: ::std::error::Error }),
            );

    let where_clause = augment_where_clause(where_clause, extra_predicates);

    let error_trait = spanned_error_trait(input.original);
    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #error_trait for #ty #ty_generics #where_clause {
            #source_method
            #backtrace_method
        }
        #display_impl
        #from_impl
    }
}

fn impl_enum(input: Enum) -> TokenStream {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let source_method = if input.has_source() {
        let arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            if variant.attrs.transparent.is_some() {
                let only_field = &variant.fields[0].member;
                let source = quote!(std::error::Error::source(transparent.as_dyn_error()));
                quote! {
                    #ty::#ident {#only_field: transparent} => #source,
                }
            } else if let Some(source_field) = variant.source_field() {
                let source = &source_field.member;
                let asref = if type_is_option(source_field.ty) {
                    Some(quote_spanned!(source.span()=> .as_ref()?))
                } else {
                    None
                };
                let varsource = quote!(source);
                let dyn_error = quote_spanned!(source.span()=> #varsource #asref.as_dyn_error());
                quote! {
                    #ty::#ident {#source: #varsource, ..} => std::option::Option::Some(#dyn_error),
                }
            } else {
                quote! {
                    #ty::#ident {..} => std::option::Option::None,
                }
            }
        });
        Some(quote! {
            fn source(&self) -> std::option::Option<&(dyn std::error::Error + 'static)> {
                use thiserror::private::AsDynError;
                #[allow(deprecated)]
                match self {
                    #(#arms)*
                }
            }
        })
    } else {
        None
    };

    let backtrace_method = if input.has_backtrace() {
        let arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            match (variant.backtrace_field(), variant.source_field()) {
                (Some(backtrace_field), Some(source_field))
                    if backtrace_field.attrs.backtrace.is_none() =>
                {
                    let backtrace = &backtrace_field.member;
                    let source = &source_field.member;
                    let varsource = quote!(source);
                    let source_backtrace = if type_is_option(source_field.ty) {
                        quote_spanned! {source.span()=>
                            #varsource.as_ref().and_then(|source| source.as_dyn_error().backtrace())
                        }
                    } else {
                        quote_spanned! {source.span()=>
                            #varsource.as_dyn_error().backtrace()
                        }
                    };
                    let combinator = if type_is_option(backtrace_field.ty) {
                        quote! {
                            #source_backtrace.or(backtrace.as_ref())
                        }
                    } else {
                        quote! {
                            std::option::Option::Some(#source_backtrace.unwrap_or(backtrace))
                        }
                    };
                    quote! {
                        #ty::#ident {
                            #backtrace: backtrace,
                            #source: #varsource,
                            ..
                        } => {
                            use thiserror::private::AsDynError;
                            #combinator
                        }
                    }
                }
                (Some(backtrace_field), _) => {
                    let backtrace = &backtrace_field.member;
                    let body = if type_is_option(backtrace_field.ty) {
                        quote!(backtrace.as_ref())
                    } else {
                        quote!(std::option::Option::Some(backtrace))
                    };
                    quote! {
                        #ty::#ident {#backtrace: backtrace, ..} => #body,
                    }
                }
                (None, _) => quote! {
                    #ty::#ident {..} => std::option::Option::None,
                },
            }
        });
        Some(quote! {
            fn backtrace(&self) -> std::option::Option<&std::backtrace::Backtrace> {
                #[allow(deprecated)]
                match self {
                    #(#arms)*
                }
            }
        })
    } else {
        None
    };

    let display_impl = if input.has_display() {
        let use_as_display = if input.variants.iter().any(|v| {
            v.attrs
                .display
                .as_ref()
                .map_or(false, |display| display.has_bonus_display)
        }) {
            Some(quote! {
                #[allow(unused_imports)]
                use thiserror::private::{DisplayAsDisplay, PathAsDisplay};
            })
        } else {
            None
        };
        let void_deref = if input.variants.is_empty() {
            Some(quote!(*))
        } else {
            None
        };

        let mut extra_predicates = Vec::new();
        for field in input.variants.iter().flat_map(|v| {
            v.attrs
                .display
                .iter()
                .flat_map(move |d| d.iter_fmt_types(&v.fields))
        }) {
            use crate::attr::DisplayFormatMarking;
            let (ty, bound, ast): (
                &syn::Type,
                syn::punctuated::Punctuated<syn::TypeParamBound, _>,
                &syn::Field,
            );
            match field {
                DisplayFormatMarking::Debug(f) => {
                    ty = f.ty;
                    bound = parse_quote! { ::std::fmt::Debug };
                    ast = f.original;
                }
                DisplayFormatMarking::Display(f) => {
                    ty = f.ty;
                    bound = parse_quote! { ::std::fmt::Display };
                    ast = f.original;
                }
            }
            // If a generic is at all present, a constraint will be applied to the field type
            // This may create redundant `AlwaysDebug<T>: Debug` scenarios, but covers T: Debug and &T: Debug cleanly
            let mut usages = GenericUsageVisitor::new_unmarked(
                input.generics.type_params().map(|p| p.ident.clone()),
            );
            syn::visit::visit_field(&mut usages, ast);
            if usages.iter_marked().next().is_some() {
                extra_predicates.push(syn::WherePredicate::Type(syn::PredicateType {
                    bounded_ty: ty.clone(),
                    colon_token: syn::token::Colon::default(),
                    bounds: bound,
                    lifetimes: None,
                }));
            }
        }

        let arms = input.variants.iter().map(|variant| {
            let display = match &variant.attrs.display {
                Some(display) => display.to_token_stream(),
                None => {
                    let only_field = match &variant.fields[0].member {
                        Member::Named(ident) => ident.clone(),
                        Member::Unnamed(index) => format_ident!("_{}", index),
                    };
                    quote!(std::fmt::Display::fmt(#only_field, __formatter))
                }
            };
            let ident = &variant.ident;
            let pat = fields_pat(&variant.fields);
            quote! {
                #ty::#ident #pat => #display
            }
        });

        let where_clause = augment_where_clause(where_clause, extra_predicates);

        Some(quote! {
            #[allow(unused_qualifications)]
            impl #impl_generics std::fmt::Display for #ty #ty_generics #where_clause {
                fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    #use_as_display
                    #[allow(
                        unused_variables,
                        deprecated,
                        // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
                        clippy::nonstandard_macro_braces,
                        clippy::used_underscore_binding,
                    )]
                    match #void_deref self {
                        #(#arms,)*
                    }
                }
            }
        })
    } else {
        None
    };

    let from_impls = input.variants.iter().filter_map(|variant| {
        let from_field = variant.from_field()?;
        let backtrace_field = variant.backtrace_field();
        let variant = &variant.ident;
        let from = from_field.ty;
        let body = from_initializer(from_field, backtrace_field);
        Some(quote! {
            #[allow(unused_qualifications)]
            impl #impl_generics std::convert::From<#from> for #ty #ty_generics #where_clause {
                #[allow(deprecated)]
                fn from(source: #from) -> Self {
                    #ty::#variant #body
                }
            }
        })
    });

    let (generic_field_types, generics_in_from_types): (Vec<&syn::Type>, Vec<proc_macro2::Ident>) = {
        let mut generics_in_from_types = GenericUsageVisitor::new_unmarked(
            input.generics.type_params().map(|p| p.ident.clone()),
        );
        let mut generic_field_types = Vec::new();
        for from_field in input
            .variants
            .iter()
            .filter_map(|variant| variant.from_field())
        {
            let mut generics_in_this_field = GenericUsageVisitor::new_unmarked(
                input.generics.type_params().map(|p| p.ident.clone()),
            );
            syn::visit::visit_type(&mut generics_in_this_field, &from_field.original.ty);
            if generics_in_from_types.mark_from(generics_in_this_field.iter_marked()) {
                generic_field_types.push(from_field.ty);
            }
        }
        (
            generic_field_types,
            generics_in_from_types.generics.into_keys().collect(),
        )
    };

    let extra_predicates =
        std::iter::once(parse_quote! { Self: ::std::fmt::Display + ::std::fmt::Debug })
            .chain(
                generics_in_from_types
                    .into_iter()
                    .map(|generic| parse_quote! { #generic: 'static }),
            )
            .chain(
                generic_field_types
                    .into_iter()
                    .map(|ty| parse_quote! { #ty: ::std::error::Error }),
            );

    let where_clause = augment_where_clause(where_clause, extra_predicates);

    let error_trait = spanned_error_trait(input.original);
    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics #error_trait for #ty #ty_generics #where_clause {
            #source_method
            #backtrace_method
        }
        #display_impl
        #(#from_impls)*
    }
}

#[cfg_attr(test, derive(Debug))]
#[derive(Clone)]
struct GenericUsageVisitor {
    generics: std::collections::HashMap<proc_macro2::Ident, bool>,
}

impl GenericUsageVisitor {
    pub fn new<TPairs>(generics: TPairs) -> Self
    where
        TPairs: IntoIterator<Item = (syn::Ident, bool)>,
    {
        Self {
            generics: generics.into_iter().collect(),
        }
    }

    pub fn new_unmarked<TIdents>(generics: TIdents) -> Self
    where
        TIdents: IntoIterator<Item = syn::Ident>,
    {
        Self::new(generics.into_iter().map(|ident| (ident, false)))
    }

    pub fn iter_marked(&self) -> impl Iterator<Item = &proc_macro2::Ident> {
        self.generics.iter().filter(|(_k, v)| **v).map(|(k, _v)| k)
    }

    #[allow(dead_code)]
    pub fn is_marked<'a, T: PartialEq<&'a proc_macro2::Ident> + 'a>(&'a self, item: T) -> bool {
        self.iter_marked().any(|ident| item == ident)
    }

    pub fn mark_from<'a, TMarkedSource: IntoIterator<Item = &'a proc_macro2::Ident> + 'a>(
        &'a mut self,
        source: TMarkedSource,
    ) -> bool {
        let mut any_marked = false;
        for other_marked in source {
            if let Some(is_marked) = self.generics.get_mut(other_marked) {
                *is_marked = true;
                any_marked = true;
            }
        }
        any_marked
    }
}

impl<'ast> syn::visit::Visit<'ast> for GenericUsageVisitor {
    fn visit_type_path(&mut self, i: &'ast syn::TypePath) {
        if let Some(ident) = i.path.get_ident() {
            if let Some(entry) = self.generics.get_mut(ident) {
                *entry = true;
            }
        }
        syn::visit::visit_type_path(self, i);
    }

    fn visit_type_param(&mut self, i: &'ast syn::TypeParam) {
        if let Some(entry) = self.generics.get_mut(&i.ident) {
            *entry = true;
        }
        syn::visit::visit_type_param(self, i);
    }
}

#[cfg(test)]
mod test_visitors {
    use proc_macro2::Span;

    use crate::expand::GenericUsageVisitor;

    #[test]
    fn test_generic_usage_visitor() {
        fn ident_for<'a>(ident_str: &'a str) -> syn::Ident {
            syn::Ident::new(ident_str, Span::call_site())
        }

        let field: syn::Variant = syn::parse_quote! { X(Foo<Bar>) };

        let mut visitor = GenericUsageVisitor::new(
            vec!["Bar", "Baz"]
                .into_iter()
                .map(|ident_str| (ident_for(ident_str), false)),
        );
        syn::visit::visit_variant(&mut visitor, &field);
        assert_eq!(
            visitor.generics.get(&ident_for("Bar")),
            Some(&true),
            "Bar must be marked"
        );
        assert_eq!(
            visitor.generics.get(&ident_for("Baz")),
            Some(&false),
            "Baz must be present but not marked"
        );
    }
}

fn augment_where_clause<TPredicates>(
    where_clause: Option<&syn::WhereClause>,
    extra_predicates: TPredicates,
) -> Option<syn::WhereClause>
where
    TPredicates: IntoIterator<Item = syn::WherePredicate>,
{
    let mut extra_predicates = extra_predicates.into_iter().peekable();
    if extra_predicates.peek().is_none() {
        return where_clause.cloned();
    }
    Some(match where_clause {
        Some(w) => syn::WhereClause {
            where_token: w.where_token,
            predicates: w
                .predicates
                .iter()
                .cloned()
                .chain(extra_predicates)
                .collect(),
        },
        None => syn::WhereClause {
            where_token: syn::token::Where::default(),
            predicates: extra_predicates.into_iter().collect(),
        },
    })
}

fn fields_pat(fields: &[Field]) -> TokenStream {
    let mut members = fields.iter().map(|field| &field.member).peekable();
    match members.peek() {
        Some(Member::Named(_)) => quote!({ #(#members),* }),
        Some(Member::Unnamed(_)) => {
            let vars = members.map(|member| match member {
                Member::Unnamed(member) => format_ident!("_{}", member),
                Member::Named(_) => unreachable!(),
            });
            quote!((#(#vars),*))
        }
        None => quote!({}),
    }
}

fn from_initializer(from_field: &Field, backtrace_field: Option<&Field>) -> TokenStream {
    let from_member = &from_field.member;
    let backtrace = backtrace_field.map(|backtrace_field| {
        let backtrace_member = &backtrace_field.member;
        if type_is_option(backtrace_field.ty) {
            quote! {
                #backtrace_member: std::option::Option::Some(std::backtrace::Backtrace::capture()),
            }
        } else {
            quote! {
                #backtrace_member: std::convert::From::from(std::backtrace::Backtrace::capture()),
            }
        }
    });
    quote!({
        #from_member: source,
        #backtrace
    })
}

fn type_is_option(ty: &Type) -> bool {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return false,
    };

    let last = path.segments.last().unwrap();
    if last.ident != "Option" {
        return false;
    }

    match &last.arguments {
        PathArguments::AngleBracketed(bracketed) => bracketed.args.len() == 1,
        _ => false,
    }
}

fn spanned_error_trait(input: &DeriveInput) -> TokenStream {
    let vis_span = match &input.vis {
        Visibility::Public(vis) => Some(vis.pub_token.span()),
        Visibility::Crate(vis) => Some(vis.crate_token.span()),
        Visibility::Restricted(vis) => Some(vis.pub_token.span()),
        Visibility::Inherited => None,
    };
    let data_span = match &input.data {
        Data::Struct(data) => data.struct_token.span(),
        Data::Enum(data) => data.enum_token.span(),
        Data::Union(data) => data.union_token.span(),
    };
    let first_span = vis_span.unwrap_or(data_span);
    let last_span = input.ident.span();
    let path = quote_spanned!(first_span=> std::error::);
    let error = quote_spanned!(last_span=> Error);
    quote!(#path #error)
}

use crate::ast::{Enum, Field, Input, Modifier, Struct};
use crate::attr::{Attrs, Trait};
use crate::fallback;
use crate::generics::{InferredBounds, ParamsInScope};
use crate::unraw::MemberUnraw;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::parse::Parse;
use std::collections::BTreeSet as Set;
use std::fmt::Debug;
use syn::{
    Data, DeriveInput, GenericArgument, PathArguments, Result, Token, Type, TypePath, parse, parse_macro_input, parse_quote
};

pub fn derive(input: &DeriveInput, typical: bool) -> TokenStream {
    match try_expand(input, typical) {
        Ok(expanded) => expanded,
        // If there are invalid attributes in the input, expand to an Error impl
        // anyway to minimize spurious secondary errors in other code that uses
        // this type as an Error.
        Err(error) => fallback::expand(input, error),
    }
}

fn try_expand(input: &DeriveInput, typical: bool) -> Result<TokenStream> {
    let input = Input::from_syn(input)?;
    input.validate()?;
    Ok(match input {
        Input::Struct(input) => impl_struct(input, typical)?,
        Input::Enum(input) => impl_enum(input, typical),
    })
}

/// Modifies the struct and pass the work to derive macro. This is recursive
pub fn try_expand_to_derive(input: &DeriveInput, typical: bool) -> Result<TokenStream> {
    let input = Input::from_syn(input)?;
    input.validate()?;
    Ok(match input {
        Input::Struct(input) => {
            let attrs = &input.derive_input.attrs;
            let ty = call_site_ident(&input.ident);

            let attrs = &input.derive_input.attrs;
            let data_struct = input.node();
            let bt = input.backtrace_field().is_none()
                && input.from_field().is_none()
                && input.source_field().is_none();
            let fields = &data_struct.fields;
            let fields_mem = fields.iter();
            let scope = ParamsInScope::new(&input.generics);

            // Check if this is a tuple struct (unnamed fields)
            let is_tuple = fields.iter().any(|f| f.ident.is_none());

            if bt {
                let prepend: TokenStream = if is_tuple {
                    // Handle tuple struct
                    let mut new_fields = Vec::new();
                    for f in fields {
                        let fa = &f.attrs;
                        let ty = &f.ty;
                        let f1: TokenStream = parse_quote!(
                            #(#fa)*
                            #ty
                        );
                        new_fields.push(f1);
                    }
                    // Add backtrace field
                    new_fields.push(parse_quote!(crate::backtrace::Backtrace));

                    parse_quote!(
                        #[derive(Error, Debug)]
                        #(#attrs)*
                        pub struct #ty(#(#new_fields),*);
                    )
                } else {
                    // Handle regular struct
                    parse_quote!(
                        #[derive(Error, Debug)]
                        #(#attrs)*
                        pub struct #ty {
                            #(#fields_mem,)*
                            pub backtrace: crate::backtrace::Backtrace
                        }
                    )
                };
                prepend
            } else {
                if is_tuple {
                    parse_quote!(
                        #[derive(Error, Debug)]
                        #(#attrs)*
                        pub struct #ty (
                            #(#fields_mem,)*
                        );
                    )
                } else {
                    parse_quote!(
                        #[derive(Error, Debug)]
                        #(#attrs)*
                        pub struct #ty {
                            #(#fields_mem,)*
                        }
                    )
                }
            }
        }
        Input::Enum(input) => {
            let mut vars_modified = Vec::new();
            for variant in input.variants {
                let h = variant.original;
                let any_name = h.fields.iter().any(|f| f.ident.is_some());
                let tuple = !any_name;
                let fields = h.fields.iter();
                let mut new_fields = Vec::new();
                let attrs = &h.attrs;
                let name = &h.ident;

                variant.backtrace_field();
                for f in fields {
                    let name = &f.ident;
                    let fa = &f.attrs;
                    let ty = &f.ty;
                    let f1: TokenStream = match name {
                        Some(id) => {
                            parse_quote!(
                                #(#fa)*
                                #id: #ty
                            )
                        }
                        None => {
                            parse_quote!(
                                #(#fa)*
                                #ty
                            )
                        }
                    };
                    new_fields.push(f1);
                }
                let bt: TokenStream = {
                    if tuple {
                        parse_quote!(crate::backtrace::Backtrace)
                    } else {
                        parse_quote!(
                            backtrace: crate::backtrace::Backtrace
                        )
                    }
                };

                if variant.backtrace_field().is_none() && variant.attrs.transparent.is_none() {
                    new_fields.push(bt);
                }

                let display_derive: Option<TokenStream> = if variant.attrs.display.is_none() {
                    Some(parse_quote!(
                        #[error("{:?}", self)]
                    ))
                } else {
                    None
                };
                let body: TokenStream = if tuple {
                    parse_quote!(
                    (
                            #(#new_fields, )*
                    )
                        )
                } else {
                    parse_quote!({
                            #(#new_fields,)*
                        }
                    )
                };
                let modified: TokenStream = parse_quote!(
                    #display_derive
                    #(#attrs)*
                    #name #body
                );
                vars_modified.push(modified);
            }

            let attrs = &input.derive_input.attrs;
            let ty = call_site_ident(&input.ident);

            let attrs = &input.derive_input.attrs;

            parse_quote!(
                #[derive(Error, Debug)]
                #(#attrs)*
                pub enum #ty {
                    #(#vars_modified,)*
                }
            )
        }
    })
}

fn impl_struct(mut input: Struct, typical: bool) -> Result<TokenStream> {
    let attrs = &mut input.derive_input.attrs;
    attrs.push(parse_quote! {
        #[derive(Debug)]
    });

    let ty = call_site_ident(&input.ident);

    let attrs = &input.derive_input.attrs;
    let data_struct = input.node();
    let bt = input.backtrace_field().is_none()
        && input.from_field().is_none()
        && input.source_field().is_none();
    let fields = &data_struct.fields;
    let fields_mem = fields.iter();
    let scope = ParamsInScope::new(&input.generics);

    let prepend = if bt {
        let prepend: TokenStream = parse_quote!(
            #[derive(Error)]
            #(#attrs)*
            pub struct #ty {
                #(#fields_mem,)*
                backtrace: ::backtrace::Backtrace
            }
        );
        input.modifier.add_default_backtrace = true;

        Some(prepend)
    } else {
        None
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut error_inferred_bounds = InferredBounds::new();

    let source_body = if let Some(transparent_attr) = &input.attrs.transparent {
        let only_field = &input.fields[0];
        if only_field.contains_generic {
            error_inferred_bounds.insert(only_field.ty, quote!(::thiserror::__private::Error));
        }
        let member = &only_field.member;
        Some(quote_spanned! {transparent_attr.span=>
            ::thiserror::__private::Error::source(self.#member.as_dyn_error())
        })
    } else if let Some(source_field) = input.source_field() {
        let source = &source_field.member;
        if source_field.contains_generic {
            let ty = unoptional_type(source_field.ty);
            error_inferred_bounds.insert(ty, quote!(::thiserror::__private::Error + 'static));
        }
        let asref = if type_is_option(source_field.ty) {
            Some(quote_spanned!(source.span()=> .as_ref()?))
        } else {
            None
        };
        let dyn_error = quote_spanned! {source_field.source_span()=>
            self.#source #asref.as_dyn_error()
        };
        Some(quote! {
            ::core::option::Option::Some(#dyn_error)
        })
    } else {
        None
    };
    let source_method = source_body.map(|body| {
        quote! {
            fn source(&self) -> ::core::option::Option<&(dyn ::thiserror::__private::Error + 'static)> {
                use ::thiserror::__private::AsDynError as _;
                #body
            }
        }
    });

    let mut provide_method = input.backtrace_field().map(|backtrace_field| {
        let request = quote!(request);
        let backtrace = &backtrace_field.member;
        let body = if let Some(source_field) = input.source_field() {
            let source = &source_field.member;
            let source_provide = if type_is_option(source_field.ty) {
                quote_spanned! {source.span()=>
                    if let ::core::option::Option::Some(source) = &self.#source {
                        source.thiserror_provide(#request);
                    }
                }
            } else {
                quote_spanned! {source.span()=>
                    self.#source.thiserror_provide(#request);
                }
            };
            let self_provide = if source == backtrace {
                None
            } else if type_is_option(backtrace_field.ty) {
                Some(quote! {
                    if let ::core::option::Option::Some(backtrace) = &self.#backtrace {
                        #request.provide_ref::<::thiserror::__private::Backtrace>(backtrace);
                    }
                })
            } else {
                Some(quote! {
                    #request.provide_ref::<::thiserror::__private::Backtrace>(&self.#backtrace);
                })
            };
            quote! {
                use ::thiserror::__private::ThiserrorProvide as _;
                #source_provide
                #self_provide
            }
        } else if type_is_option(backtrace_field.ty) {
            quote! {
                if let ::core::option::Option::Some(backtrace) = &self.#backtrace {
                    #request.provide_ref::<::thiserror::__private::Backtrace>(backtrace);
                }
            }
        } else {
            quote! {
                #request.provide_ref::<::thiserror::__private::Backtrace>(&self.#backtrace);
            }
        };
        quote! {
            fn provide<'_request>(&'_request self, #request: &mut ::core::error::Request<'_request>) {
                #body
            }
        }
    });

    if input.modifier.add_default_backtrace {
        let request = quote!(request);
        let backtrace = "backtrace";
        let body = quote! {
            #request.provide_ref::<::thiserror::__private::Backtrace>(&self.#backtrace);
        };
        if let Some(_) = &provide_method {
            provide_method = Some(quote! {
                fn provide<'_request>(&'_request self, #request: &mut ::core::error::Request<'_request>) {
                    #body
                }
            });
        }
    }

    let mut display_implied_bounds = Set::new();
    let display_body = if input.attrs.transparent.is_some() {
        let only_field = &input.fields[0].member;
        display_implied_bounds.insert((0, Trait::Display));
        Some(quote! {
            ::core::fmt::Display::fmt(&self.#only_field, __formatter)
        })
    } else {
        let pat = fields_pat(&input.fields);

        Some(match &input.attrs.display {
            Some(display) => {
                display_implied_bounds.clone_from(&display.implied_bounds);
                let use_as_display = use_as_display(display.has_bonus_display);

                quote! {
                    #use_as_display
                    #[allow(unused_variables, deprecated)]
                    let Self #pat = self;
                    #display
                }
            }
            // Sane and handy default behavior.
            None => parse_quote! {std::fmt::Debug::fmt(&self, fmt)},
        })
    };
    let display_impl = display_body.map(|body| {
        let mut display_inferred_bounds = InferredBounds::new();
        for (field, bound) in display_implied_bounds {
            let field = &input.fields[field];
            if field.contains_generic {
                display_inferred_bounds.insert(field.ty, bound);
            }
        }
        let display_where_clause = display_inferred_bounds.augment_where_clause(input.generics);
        quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl #impl_generics ::core::fmt::Display for #ty #ty_generics #display_where_clause {
                #[allow(clippy::used_underscore_binding)]
                fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    #body
                }
            }
        }
    });

    let from_impl = input.from_field().map(|from_field| {
        let span = from_field.attrs.from.unwrap().span;
        let backtrace_field = input.distinct_backtrace_field();
        let from = unoptional_type(from_field.ty);
        let source_var = Ident::new("source", span);
        let body = from_initializer(from_field, backtrace_field, &source_var, &input.modifier);
        let from_function = quote! {
            fn from(#source_var: #from) -> Self {
                #ty #body
            }
        };
        let from_impl = quote_spanned! {span=>
            #[automatically_derived]
            impl #impl_generics ::core::convert::From<#from> for #ty #ty_generics #where_clause {
                #from_function
            }
        };
        Some(quote! {
            #[allow(
                deprecated,
                unused_qualifications,
                clippy::elidable_lifetime_names,
                clippy::needless_lifetimes,
            )]
            #from_impl
        })
    });

    if input.generics.type_params().next().is_some() {
        let self_token = <Token![Self]>::default();
        error_inferred_bounds.insert(self_token, Trait::Debug);
        error_inferred_bounds.insert(self_token, Trait::Display);
    }
    let error_where_clause = error_inferred_bounds.augment_where_clause(input.generics);

    Ok(quote! {
        #prepend

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::thiserror::__private::Error for #ty #ty_generics #error_where_clause {
            #source_method
            #provide_method
        }
        #display_impl
        #from_impl
    })
}

fn impl_enum(input: Enum, typical: bool) -> TokenStream {
    let ty = call_site_ident(&input.ident);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut error_inferred_bounds = InferredBounds::new();

    let source_method = if input.has_source() {
        let arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            if let Some(transparent_attr) = &variant.attrs.transparent {
                let only_field = &variant.fields[0];
                if only_field.contains_generic {
                    error_inferred_bounds.insert(only_field.ty, quote!(::thiserror::__private::Error));
                }
                let member = &only_field.member;
                let source = quote_spanned! {transparent_attr.span=>
                    ::thiserror::__private::Error::source(transparent.as_dyn_error())
                };
                quote! {
                    #ty::#ident {#member: transparent} => #source,
                }
            } else if let Some(source_field) = variant.source_field() {
                let source = &source_field.member;
                if source_field.contains_generic {
                    let ty = unoptional_type(source_field.ty);
                    error_inferred_bounds.insert(ty, quote!(::thiserror::__private::Error + 'static));
                }
                let asref = if type_is_option(source_field.ty) {
                    Some(quote_spanned!(source.span()=> .as_ref()?))
                } else {
                    None
                };
                let varsource = quote!(source);
                let dyn_error = quote_spanned! {source_field.source_span()=>
                    #varsource #asref.as_dyn_error()
                };
                quote! {
                    #ty::#ident {#source: #varsource, ..} => ::core::option::Option::Some(#dyn_error),
                }
            } else {
                quote! {
                    #ty::#ident {..} => ::core::option::Option::None,
                }
            }
        });
        Some(quote! {
            fn source(&self) -> ::core::option::Option<&(dyn ::thiserror::__private::Error + 'static)> {
                use ::thiserror::__private::AsDynError as _;
                #[allow(deprecated)]
                match self {
                    #(#arms)*
                }
            }
        })
    } else {
        None
    };

    let provide_method = if input.has_backtrace() {
        let request = quote!(request);
        let arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            match (variant.backtrace_field(), variant.source_field()) {
                (Some(backtrace_field), Some(source_field))
                    if backtrace_field.attrs.backtrace.is_none() =>
                {
                    let backtrace = &backtrace_field.member;
                    let source = &source_field.member;
                    let varsource = quote!(source);
                    let source_provide = if type_is_option(source_field.ty) {
                        quote_spanned! {source.span()=>
                            if let ::core::option::Option::Some(source) = #varsource {
                                source.thiserror_provide(#request);
                            }
                        }
                    } else {
                        quote_spanned! {source.span()=>
                            #varsource.thiserror_provide(#request);
                        }
                    };
                    let self_provide = if type_is_option(backtrace_field.ty) {
                        quote! {
                            if let ::core::option::Option::Some(backtrace) = backtrace {
                                #request.provide_ref::<::thiserror::__private::Backtrace>(backtrace);
                            }
                        }
                    } else {
                        quote! {
                            #request.provide_ref::<::thiserror::__private::Backtrace>(backtrace);
                        }
                    };
                    quote! {
                        #ty::#ident {
                            #backtrace: backtrace,
                            #source: #varsource,
                            ..
                        } => {
                            use ::thiserror::__private::ThiserrorProvide as _;
                            #source_provide
                            #self_provide
                        }
                    }
                }
                (Some(backtrace_field), Some(source_field))
                    if backtrace_field.member == source_field.member =>
                {
                    let backtrace = &backtrace_field.member;
                    let varsource = quote!(source);
                    let source_provide = if type_is_option(source_field.ty) {
                        quote_spanned! {backtrace.span()=>
                            if let ::core::option::Option::Some(source) = #varsource {
                                source.thiserror_provide(#request);
                            }
                        }
                    } else {
                        quote_spanned! {backtrace.span()=>
                            #varsource.thiserror_provide(#request);
                        }
                    };
                    quote! {
                        #ty::#ident {#backtrace: #varsource, ..} => {
                            use ::thiserror::__private::ThiserrorProvide as _;
                            #source_provide
                        }
                    }
                }
                (Some(backtrace_field), _) => {
                    let backtrace = &backtrace_field.member;
                    let body = if type_is_option(backtrace_field.ty) {
                        quote! {
                            if let ::core::option::Option::Some(backtrace) = backtrace {
                                #request.provide_ref::<::thiserror::__private::Backtrace>(backtrace);
                            }
                        }
                    } else {
                        quote! {
                            #request.provide_ref::<::thiserror::__private::Backtrace>(backtrace);
                        }
                    };
                    quote! {
                        #ty::#ident {#backtrace: backtrace, ..} => {
                            #body
                        }
                    }
                }
                (None, _) => quote! {
                    #ty::#ident {..} => {}
                },
            }
        });
        Some(quote! {
            fn provide<'_request>(&'_request self, #request: &mut ::core::error::Request<'_request>) {
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
        let mut display_inferred_bounds = InferredBounds::new();
        let has_bonus_display = input.variants.iter().any(|v| {
            v.attrs
                .display
                .as_ref()
                .map_or(false, |display| display.has_bonus_display)
        });
        let use_as_display = use_as_display(has_bonus_display);
        let void_deref = if input.variants.is_empty() {
            Some(quote!(*))
        } else {
            None
        };
        let arms = input.variants.iter().map(|variant| {
            let mut display_implied_bounds = Set::new();
            let display = if let Some(display) = &variant.attrs.display {
                display_implied_bounds.clone_from(&display.implied_bounds);
                display.to_token_stream()
            } else if let Some(fmt) = &variant.attrs.fmt {
                let fmt_path = &fmt.path;
                let vars = variant.fields.iter().map(|field| match &field.member {
                    MemberUnraw::Named(ident) => ident.to_local(),
                    MemberUnraw::Unnamed(index) => format_ident!("_{}", index),
                });
                quote!(#fmt_path(#(#vars,)* __formatter))
            } else {
                let only_field = match &variant.fields[0].member {
                    MemberUnraw::Named(ident) => ident.to_local(),
                    MemberUnraw::Unnamed(index) => format_ident!("_{}", index),
                };
                display_implied_bounds.insert((0, Trait::Display));
                quote!(::core::fmt::Display::fmt(#only_field, __formatter))
            };
            for (field, bound) in display_implied_bounds {
                let field = &variant.fields[field];
                if field.contains_generic {
                    display_inferred_bounds.insert(field.ty, bound);
                }
            }
            let ident = &variant.ident;
            let pat = fields_pat(&variant.fields);
            quote! {
                #ty::#ident #pat => #display
            }
        });
        let arms = arms.collect::<Vec<_>>();
        let display_where_clause = display_inferred_bounds.augment_where_clause(input.generics);
        Some(quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl #impl_generics ::core::fmt::Display for #ty #ty_generics #display_where_clause {
                fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    #use_as_display
                    #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
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
        let span = from_field.attrs.from.unwrap().span;
        let backtrace_field = variant.distinct_backtrace_field();
        let variant = &variant.ident;
        let from = unoptional_type(from_field.ty);
        let source_var = Ident::new("source", span);
        let body = from_initializer(from_field, backtrace_field, &source_var, &input.modifier);
        let from_function = quote! {
            fn from(#source_var: #from) -> Self {
                #ty::#variant #body
            }
        };
        let from_impl = quote_spanned! {span=>
            #[automatically_derived]
            impl #impl_generics ::core::convert::From<#from> for #ty #ty_generics #where_clause {
                #from_function
            }
        };
        Some(quote! {
            #[allow(
                deprecated,
                unused_qualifications,
                clippy::elidable_lifetime_names,
                clippy::needless_lifetimes,
            )]
            #from_impl
        })
    });

    if input.generics.type_params().next().is_some() {
        let self_token = <Token![Self]>::default();
        error_inferred_bounds.insert(self_token, Trait::Debug);
        error_inferred_bounds.insert(self_token, Trait::Display);
    }
    let error_where_clause = error_inferred_bounds.augment_where_clause(input.generics);

    quote! {
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::thiserror::__private::Error for #ty #ty_generics #error_where_clause {
            #source_method
            #provide_method
        }
        #display_impl
        #(#from_impls)*
    }
}

// Create an ident with which we can expand `impl Trait for #ident {}` on a
// deprecated type without triggering deprecation warning on the generated impl.
pub(crate) fn call_site_ident(ident: &Ident) -> Ident {
    let mut ident = ident.clone();
    ident.set_span(ident.span().resolved_at(Span::call_site()));
    ident
}

fn fields_pat(fields: &[Field]) -> TokenStream {
    let mut members = fields.iter().map(|field| &field.member).peekable();
    match members.peek() {
        Some(MemberUnraw::Named(_)) => quote!({ #(#members),* }),
        Some(MemberUnraw::Unnamed(_)) => {
            let vars = members.map(|member| match member {
                MemberUnraw::Unnamed(index) => format_ident!("_{}", index),
                MemberUnraw::Named(_) => unreachable!(),
            });
            quote!((#(#vars),*))
        }
        None => quote!({}),
    }
}

fn use_as_display(needs_as_display: bool) -> Option<TokenStream> {
    if needs_as_display {
        Some(quote! {
            use ::thiserror::__private::AsDisplay as _;
        })
    } else {
        None
    }
}

fn from_initializer(
    from_field: &Field,
    backtrace_field: Option<&Field>,
    source_var: &Ident,
    modifier: &Modifier,
) -> TokenStream {
    let from_member = &from_field.member;
    let some_source = if type_is_option(from_field.ty) {
        quote!(::core::option::Option::Some(#source_var))
    } else {
        quote!(#source_var)
    };
    let backtrace = backtrace_field.map(|backtrace_field| {
        let backtrace_member = &backtrace_field.member;
        if type_is_option(backtrace_field.ty) {
            quote! {
                #backtrace_member: ::core::option::Option::Some(::thiserror::__private::Backtrace::new()),
            }
        } else {
            quote! {
                #backtrace_member: ::core::convert::From::from(::thiserror::__private::Backtrace::new()),
            }
        }
    }).or_else(
        || if modifier.add_default_backtrace {Some(
            quote! {
                backtrace: ::backtrace::Backtrace::new()
            }
        )} else {None}
    );
    quote!({
        #from_member: #some_source,
        #backtrace
    })
}

fn type_is_option(ty: &Type) -> bool {
    type_parameter_of_option(ty).is_some()
}

fn unoptional_type(ty: &Type) -> TokenStream {
    let unoptional = type_parameter_of_option(ty).unwrap_or(ty);
    quote!(#unoptional)
}

fn type_parameter_of_option(ty: &Type) -> Option<&Type> {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return None,
    };

    let last = path.segments.last().unwrap();
    if last.ident != "Option" {
        return None;
    }

    let bracketed = match &last.arguments {
        PathArguments::AngleBracketed(bracketed) => bracketed,
        _ => return None,
    };

    if bracketed.args.len() != 1 {
        return None;
    }

    match &bracketed.args[0] {
        GenericArgument::Type(arg) => Some(arg),
        _ => None,
    }
}

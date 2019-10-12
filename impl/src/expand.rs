use crate::ast::{Enum, Field, Input, Struct};
use crate::attr::Attrs;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DeriveInput, Error, Member, Result, Type};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    let input = Input::from_syn(node)?;
    match input {
        Input::Struct(input) => impl_struct(input),
        Input::Enum(input) => impl_enum(input),
    }
}

fn impl_struct(input: Struct) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let source = source_member(&input.fields);
    let source_method = source.map(|source| {
        let member = quote_spanned!(source.span()=> self.#source);
        quote! {
            fn source(&self) -> std::option::Option<&(dyn std::error::Error + 'static)> {
                use thiserror::private::AsDynError;
                std::option::Option::Some(#member.as_dyn_error())
            }
        }
    });

    let backtrace = backtrace_member(&input.fields);
    let backtrace_method = backtrace.map(|backtrace| {
        quote! {
            fn backtrace(&self) -> std::option::Option<&std::backtrace::Backtrace> {
                std::option::Option::Some(&self.#backtrace)
            }
        }
    });

    let display = input.attrs.display.as_ref().map(|display| {
        let pat = fields_pat(&input.fields);
        quote! {
            impl #impl_generics std::fmt::Display for #ty #ty_generics #where_clause {
                fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    #[allow(unused_variables)]
                    let Self #pat = self;
                    #display
                }
            }
        }
    });

    Ok(quote! {
        impl #impl_generics std::error::Error for #ty #ty_generics #where_clause {
            #source_method
            #backtrace_method
        }
        #display
    })
}

fn impl_enum(input: Enum) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let sources: Vec<Option<&Member>> = input
        .variants
        .iter()
        .map(|variant| source_member(&variant.fields))
        .collect();

    let backtraces: Vec<Option<&Member>> = input
        .variants
        .iter()
        .map(|variant| backtrace_member(&variant.fields))
        .collect();

    let source_method = if sources.iter().any(Option::is_some) {
        let arms = input.variants.iter().zip(sources).map(|(variant, source)| {
            let ident = &variant.ident;
            match source {
                Some(source) => quote! {
                    #ty::#ident {#source: source, ..} => std::option::Option::Some(source.as_dyn_error()),
                },
                None => quote! {
                    #ty::#ident {..} => std::option::Option::None,
                },
            }
        });
        Some(quote! {
            fn source(&self) -> std::option::Option<&(dyn std::error::Error + 'static)> {
                use thiserror::private::AsDynError;
                match self {
                    #(#arms)*
                }
            }
        })
    } else {
        None
    };

    let backtrace_method = if backtraces.iter().any(Option::is_some) {
        let arms = input.variants.iter().zip(backtraces).map(|(variant, backtrace)| {
            let ident = &variant.ident;
            match backtrace {
                Some(backtrace) => quote! {
                    #ty::#ident {#backtrace: backtrace, ..} => std::option::Option::Some(backtrace),
                },
                None => quote! {
                    #ty::#ident {..} => std::option::Option::None,
                },
            }
        });
        Some(quote! {
            fn backtrace(&self) -> std::option::Option<&std::backtrace::Backtrace> {
                match self {
                    #(#arms)*
                }
            }
        })
    } else {
        None
    };

    let variant_attrs: Vec<&Attrs> = input
        .variants
        .iter()
        .map(|variant| &variant.attrs)
        .collect();
    let display = if variant_attrs.iter().any(|attrs| attrs.display.is_some()) {
        let arms = input
            .variants
            .iter()
            .zip(variant_attrs)
            .map(|(variant, attrs)| {
                let display = attrs.display.as_ref().ok_or_else(|| {
                    Error::new_spanned(
                        &variant.ident,
                        "missing #[error(\"...\")] display attribute",
                    )
                })?;
                let ident = &variant.ident;
                let pat = fields_pat(&variant.fields);
                Ok(quote! {
                    #ty::#ident #pat => #display
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Some(quote! {
            impl #impl_generics std::fmt::Display for #ty #ty_generics #where_clause {
                fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    #[allow(unused_variables)]
                    match self {
                        #(#arms,)*
                    }
                }
            }
        })
    } else {
        None
    };

    Ok(quote! {
        impl #impl_generics std::error::Error for #ty #ty_generics #where_clause {
            #source_method
            #backtrace_method
        }
        #display
    })
}

fn source_member<'a>(fields: &'a [Field]) -> Option<&'a Member> {
    for field in fields {
        if field.attrs.source {
            return Some(&field.member);
        }
    }
    None
}

fn backtrace_member<'a>(fields: &'a [Field]) -> Option<&'a Member> {
    for field in fields {
        if type_is_backtrace(&field.ty) {
            return Some(&field.member);
        }
    }
    None
}

fn type_is_backtrace(ty: &Type) -> bool {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return false,
    };

    let last = path.segments.last().unwrap();
    last.ident == "Backtrace" && last.arguments.is_empty()
}

fn fields_pat(fields: &[Field]) -> TokenStream {
    let mut members = fields.iter().map(|field| &field.member).peekable();
    match members.peek() {
        Some(Member::Named(_)) => quote!({ #(#members),* }),
        Some(Member::Unnamed(_)) => {
            let vars = members.map(|member| match member {
                Member::Unnamed(member) => format_ident!("_{}", member.index),
                Member::Named(_) => unreachable!(),
            });
            quote!((#(#vars),*))
        }
        None => quote!({}),
    }
}

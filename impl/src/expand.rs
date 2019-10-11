use crate::attr;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Error, Field, Fields, Ident, Index, Member, Result,
    Type,
};

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(data) => impl_struct(input, data),
        Data::Enum(data) => impl_enum(input, data),
        Data::Union(_) => Err(Error::new_spanned(
            input,
            "union as errors are not supported",
        )),
    }
}

fn impl_struct(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let source = match &data.fields {
        Fields::Named(fields) => source_member(&fields.named)?,
        Fields::Unnamed(fields) => source_member(&fields.unnamed)?,
        Fields::Unit => None,
    };

    let from = match &data.fields {
        Fields::Named(fields) => from_member_type(&fields.named)?,
        Fields::Unnamed(fields) => from_member_type(&fields.unnamed)?,
        Fields::Unit => None,
    };

    let backtrace = match &data.fields {
        Fields::Named(fields) => backtrace_member(&fields.named)?,
        Fields::Unnamed(fields) => backtrace_member(&fields.unnamed)?,
        Fields::Unit => None,
    };

    let source_method = source.map(|source| {
        let member = quote_spanned!(source.span()=> self.#source);
        quote! {
            fn source(&self) -> std::option::Option<&(dyn std::error::Error + 'static)> {
                use thiserror::private::AsDynError;
                std::option::Option::Some(#member.as_dyn_error())
            }
        }
    });

    let backtrace_method = backtrace.map(|backtrace| {
        quote! {
            fn backtrace(&self) -> std::option::Option<&std::backtrace::Backtrace> {
                std::option::Option::Some(&self.#backtrace)
            }
        }
    });

    let display = attr::display(&input.attrs)?.map(|display| {
        let pat = match &data.fields {
            Fields::Named(fields) => {
                let var = fields.named.iter().map(|field| &field.ident);
                quote!(Self { #(#var),* })
            }
            Fields::Unnamed(fields) => {
                let var = (0..fields.unnamed.len()).map(|i| format_ident!("_{}", i));
                quote!(Self(#(#var),*))
            }
            Fields::Unit => quote!(_),
        };
        quote! {
            impl #impl_generics std::fmt::Display for #ty #ty_generics #where_clause {
                fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    #[allow(unused_variables)]
                    let #pat = self;
                    #display
                }
            }
        }
    });

    let from_derive = from.map(|(from_member, from_type)| {
        let from_struct = match from_member {
            Member::Named(ident) => quote!(Self { #ident: src_err }),
            Member::Unnamed(_) => quote!(Self(src_err)),
        };

        quote! {
            impl #impl_generics std::convert::From<#from_type> for #ty #ty_generics #where_clause {
                fn from(src_err: #from_type) -> Self {
                    #from_struct
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
        #from_derive
    })
}

fn impl_enum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let sources = data
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            Fields::Named(fields) => source_member(&fields.named),
            Fields::Unnamed(fields) => source_member(&fields.unnamed),
            Fields::Unit => Ok(None),
        })
        .collect::<Result<Vec<_>>>()?;

    let froms = data
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            Fields::Named(fields) => from_member_type(&fields.named),
            Fields::Unnamed(fields) => from_member_type(&fields.unnamed),
            Fields::Unit => Ok(None),
        })
        .collect::<Result<Vec<_>>>()?;

    let backtraces = data
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            Fields::Named(fields) => backtrace_member(&fields.named),
            Fields::Unnamed(fields) => backtrace_member(&fields.unnamed),
            Fields::Unit => Ok(None),
        })
        .collect::<Result<Vec<_>>>()?;

    let source_method = if sources.iter().any(Option::is_some) {
        let arms = data.variants.iter().zip(sources).map(|(variant, source)| {
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
        let arms = data.variants.iter().zip(backtraces).map(|(variant, backtrace)| {
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

    let displays = data
        .variants
        .iter()
        .map(|variant| attr::display(&variant.attrs))
        .collect::<Result<Vec<_>>>()?;
    let display = if displays.iter().any(Option::is_some) {
        let arms = data
            .variants
            .iter()
            .zip(displays)
            .map(|(variant, display)| {
                let display = display.ok_or_else(|| {
                    Error::new_spanned(variant, "missing #[error(\"...\")] display attribute")
                })?;
                let ident = &variant.ident;
                Ok(match &variant.fields {
                    Fields::Named(fields) => {
                        let var = fields.named.iter().map(|field| &field.ident);
                        quote!(#ty::#ident { #(#var),* } => #display)
                    }
                    Fields::Unnamed(fields) => {
                        let var = (0..fields.unnamed.len()).map(|i| format_ident!("_{}", i));
                        quote!(#ty::#ident(#(#var),*) => #display)
                    }
                    Fields::Unit => quote!(#ty::#ident => #display),
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

    let froms_derive = froms
        .iter()
        .zip(data.variants.iter())
        .filter_map(|(from, variant)| {
            let variant_ident = &variant.ident;
            from.as_ref()
                .map(|(from_member, from_type)| {
                    let from_struct = match from_member {
                        Member::Named(ident) => quote!(Self::#variant_ident { #ident: src_err }),
                        Member::Unnamed(_) => quote!(Self::#variant_ident(src_err)),
                    };

                    quote! {
                        impl #impl_generics std::convert::From<#from_type> for #ty #ty_generics #where_clause {
                            fn from(src_err: #from_type) -> Self {
                                #from_struct
                            }
                        }
                    }
                })
        });


    Ok(quote! {
        impl #impl_generics std::error::Error for #ty #ty_generics #where_clause {
            #source_method
            #backtrace_method
        }
        #display
        #(#froms_derive)*
    })
}

fn source_member<'a>(fields: impl IntoIterator<Item = &'a Field>) -> Result<Option<Member>> {
    for (i, field) in fields.into_iter().enumerate() {
        if attr::is_source(field)? {
            return Ok(Some(member(i, &field.ident)));
        }
    }
    Ok(None)
}

fn from_member_type<'a>(fields: impl IntoIterator<Item = &'a Field>) -> Result<Option<(Member, Type)>> {
    for (i, field) in fields.into_iter().enumerate() {
        let is_from = attr::is_from(field)?;

        // TODO is_source can also be a field named source in the future.
        if is_from && attr::is_source(field)? {
            return Ok(Some(
                (member(i, &field.ident), field.ty.clone())
            ));
        } else if is_from {
            return Err(Error::new_spanned(field, "To derive From on this field, it must have a source (a field `source` or attr #[source])"));
        }
    }
    Ok(None)
}

fn backtrace_member<'a>(fields: impl IntoIterator<Item = &'a Field>) -> Result<Option<Member>> {
    for (i, field) in fields.into_iter().enumerate() {
        if type_is_backtrace(&field.ty) {
            return Ok(Some(member(i, &field.ident)));
        }
    }
    Ok(None)
}

fn type_is_backtrace(ty: &Type) -> bool {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return false,
    };

    let last = path.segments.last().unwrap();
    last.ident == "Backtrace" && last.arguments.is_empty()
}

fn member(i: usize, ident: &Option<Ident>) -> Member {
    match ident {
        Some(ident) => Member::Named(ident.clone()),
        None => Member::Unnamed(Index::from(i)),
    }
}

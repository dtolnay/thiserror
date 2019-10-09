use crate::attr;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Error, Field, Fields, Ident, Index, Member, Result,
    Type,
};

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(data) => struct_error(input, data),
        Data::Enum(data) => enum_error(input, data),
        Data::Union(_) => Err(Error::new_spanned(
            input,
            "union as errors are not supported",
        )),
    }
}

fn struct_error(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let source = match &data.fields {
        Fields::Named(fields) => source_member(&fields.named)?,
        Fields::Unnamed(fields) => source_member(&fields.unnamed)?,
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

    Ok(quote! {
        impl #impl_generics std::error::Error for #ident #ty_generics #where_clause {
            #source_method
            #backtrace_method
        }
    })
}

fn enum_error(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream> {
    let ident = &input.ident;
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
                    Self::#ident {#source: source, ..} => std::option::Option::Some(source.as_dyn_error()),
                },
                None => quote! {
                    Self::#ident {..} => std::option::Option::None,
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
                    Self::#ident {#backtrace: backtrace, ..} => std::option::Option::Some(backtrace),
                },
                None => quote! {
                    Self::#ident {..} => std::option::Option::None,
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

    Ok(quote! {
        impl #impl_generics std::error::Error for #ident #ty_generics #where_clause {
            #source_method
            #backtrace_method
        }
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

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

    // Because from implies source, from comes first to make sure from-specific errors
    // get encountered first
    let from = match &data.fields {
        Fields::Named(fields) => from_member_type(&fields.named)?,
        Fields::Unnamed(fields) => from_member_type(&fields.unnamed)?,
        Fields::Unit => None,
    };

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

    let backtrace_method = backtrace.as_ref().map(|backtrace| {
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
            Member::Named(ident) => quote!(Self { #ident: src_err}),
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

    let froms = data
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            Fields::Named(fields) => from_member_type(&fields.named),
            Fields::Unnamed(fields) => from_member_type(&fields.unnamed),
            Fields::Unit => Ok(None),
        })
        .collect::<Result<Vec<_>>>()?;

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

    // Best-effort to find when source types for #[from] variants are duplicated.
    // Does not take into account aliases, but will try to use as much of path as possible.
    // TODO determine if there's a better way to do the comparison loop
    for i in 0..froms.len() {
        for j in (i..froms.len()).skip(1) {
            if let Some((_, ref ty_1)) = froms[i] {
                if let Some((_, ref ty_2)) = froms[j] {

                    if ty_paths_eq(ty_1, ty_2) {
                        return Err(Error::new_spanned(&ty_2, "Cannot derive `From` on enum when two variants have same type for source error"));
                    }
                }
            }
        }
    }

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
    let mut source_member_count = 0;
    let mut res = None;

    for (i, field) in fields.into_iter().enumerate() {
        if field_is_source(&field)? {
            if  source_member_count == 1 {
                return Err(Error::new_spanned(&field.ident, "Only one `source` field allowed per struct or struct variant (remember that `#[from]` implies `source`)"));
            }

            res = Some(member(i, &field.ident));
            source_member_count += 1;
        }
    }

    Ok(res)
}

// Needs to check for conditions:
// - duplicate `from` fields
//
// `from` implies `source`
fn from_member_type<'a>(fields: impl IntoIterator<Item = &'a Field>) -> Result<Option<(Member, Type)>> {
    let mut non_from_ident= None;
    let mut res = None;

    for (i, field) in fields.into_iter().enumerate() {
        let is_from = attr::is_from(field)?;

        // Early return on errors
        if is_from {
            if res.is_some() {
                return Err(Error::new_spanned(&field.ident, "Only one `#[from]` field allowed per struct or struct variant"));
            }
        }

        // get the member
        if is_from {
            res = Some(
                (member(i, &field.ident), field.ty.clone())
            );
        } else {
            // get the latest non-member ident for error purposes
            // skip backtrace field, that's allowed to be in From impl
            if !type_is_backtrace(&field.ty) {
                non_from_ident= Some(&field.ident);
            }
        }

        // late return on this error, becaue it requires checking state at end of cycle
        if res.is_some() && non_from_ident.is_some()  {
            return Err(Error::new_spanned(non_from_ident, "When deriving `From`, non-`#[from]` fields are not allowed"));
        }
    }


    Ok(res)
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

fn ident_is_source(ident: &Option<Ident>) -> bool {
    match ident {
        Some(s) => s == "source",
        None => false,
    }
}

fn field_is_source(field: &Field) -> Result<bool> {
    Ok(attr::is_source(field)? || ident_is_source(&field.ident) || attr::is_from(field)?)
}

// currently match on as much path is available
fn ty_paths_eq(ty_1: &Type, ty_2: &Type) -> bool {
    let path_1 = match ty_1 {
        Type::Path(ty) => &ty.path,
        _ => return false,
    };
    let path_2 = match ty_2 {
        Type::Path(ty) => &ty.path,
        _ => return false,
    };

    path_1.segments.iter().rev().zip(path_2.segments.iter().rev())
        .all(|(seg_1, seg_2)| seg_1.ident == seg_2.ident)
}

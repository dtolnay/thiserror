use crate::attr;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, FieldsUnnamed, Index,
    Member, Result, Type,
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
        Fields::Named(fields) => braced_struct_source(fields)?,
        Fields::Unnamed(fields) => tuple_struct_source(fields)?,
        Fields::Unit => None,
    };

    let backtrace = match &data.fields {
        Fields::Named(fields) => braced_struct_backtrace(fields)?,
        Fields::Unnamed(fields) => tuple_struct_backtrace(fields)?,
        Fields::Unit => None,
    };

    let source_method = source.map(|source| {
        quote! {
            fn source(&self) -> std::option::Option<&(dyn std::error::Error + 'static)> {
                std::option::Option::Some(&self.#source)
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

fn braced_struct_source(fields: &FieldsNamed) -> Result<Option<Member>> {
    for field in &fields.named {
        if attr::is_source(field)? {
            return Ok(Some(Member::Named(field.ident.as_ref().unwrap().clone())));
        }
    }
    Ok(None)
}

fn tuple_struct_source(fields: &FieldsUnnamed) -> Result<Option<Member>> {
    for (i, field) in fields.unnamed.iter().enumerate() {
        if attr::is_source(field)? {
            return Ok(Some(Member::Unnamed(Index::from(i))));
        }
    }
    Ok(None)
}

fn braced_struct_backtrace(fields: &FieldsNamed) -> Result<Option<Member>> {
    for field in &fields.named {
        if type_is_backtrace(&field.ty) {
            return Ok(Some(Member::Named(field.ident.as_ref().unwrap().clone())));
        }
    }
    Ok(None)
}

fn tuple_struct_backtrace(fields: &FieldsUnnamed) -> Result<Option<Member>> {
    for (i, field) in fields.unnamed.iter().enumerate() {
        if type_is_backtrace(&field.ty) {
            return Ok(Some(Member::Unnamed(Index::from(i))));
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

fn enum_error(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream> {
    let _ = input;
    let _ = data;
    unimplemented!()
}

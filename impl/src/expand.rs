use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Error, Result, FieldsNamed, FieldsUnnamed, Fields};

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
    match &data.fields {
        Fields::Named(fields) => braced_struct_error(input, fields),
        Fields::Unnamed(fields) => tuple_struct_error(input, fields),
        Fields::Unit => unit_struct_error(input),
    }
}

fn braced_struct_error(input: &DeriveInput, fields: &FieldsNamed) -> Result<TokenStream> {
    let _ = input;
    let _ = fields;
    unimplemented!()
}

fn tuple_struct_error(input: &DeriveInput, fields: &FieldsUnnamed) -> Result<TokenStream> {
    let _ = input;
    let _ = fields;
    unimplemented!()
}

fn unit_struct_error(input: &DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    Ok(quote! {
        impl std::error::Error for #ident {}
    })
}

fn enum_error(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream> {
    let _ = input;
    let _ = data;
    unimplemented!()
}

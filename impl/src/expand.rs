use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, FieldsUnnamed, Member,
    Result,
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
        Fields::Named(fields) => braced_struct_source(input, fields)?,
        Fields::Unnamed(fields) => tuple_struct_source(input, fields)?,
        Fields::Unit => None,
    };

    let source_method = source.map(|source| {
        quote! {
            fn source(&self) -> std::option::Option<&(dyn std::error::Error + 'static)> {
                std::option::Option::Some(&self.#source)
            }
        }
    });

    Ok(quote! {
        impl #impl_generics std::error::Error for #ident #ty_generics #where_clause {
            #source_method
        }
    })
}

fn braced_struct_source(input: &DeriveInput, fields: &FieldsNamed) -> Result<Option<Member>> {
    let _ = input;
    let _ = fields;
    unimplemented!()
}

fn tuple_struct_source(input: &DeriveInput, fields: &FieldsUnnamed) -> Result<Option<Member>> {
    let _ = input;
    let _ = fields;
    unimplemented!()
}

fn enum_error(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream> {
    let _ = input;
    let _ = data;
    unimplemented!()
}

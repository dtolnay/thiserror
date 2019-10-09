use proc_macro2::TokenStream;
use syn::{DeriveInput, Result};

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let _ = input;
    unimplemented!()
}

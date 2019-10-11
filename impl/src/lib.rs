extern crate proc_macro;

mod attr;
mod expand;
mod fmt;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Error, attributes(error, source, from))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#![allow(
    clippy::blocks_in_conditions,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::enum_glob_use,
    clippy::manual_find,
    clippy::manual_let_else,
    clippy::manual_map,
    clippy::map_unwrap_or,
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::range_plus_one,
    clippy::single_match_else,
    clippy::struct_field_names,
    clippy::too_many_lines,
    clippy::wrong_self_convention
)]

extern crate proc_macro;

mod ast;
mod attr;
mod expand;
mod fallback;
mod fmt;
mod generics;
mod prop;
mod scan_expr;
mod unraw;
mod valid;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Error, attributes(backtrace, error, from, source, location))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input).into()
}

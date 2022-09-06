#![feature(error_generic_member_access, provide_any)]

use thiserror::Error;
use std::any::Provider;
use std::error::Error;

// FIXME: this should work. https://github.com/dtolnay/thiserror/issues/185
#[derive(Error, Debug)]
#[error("...")]
struct MyError {
    #[source]
    #[backtrace]
    x: std::io::Error,
}

fn main() {
    let _: dyn Error;
    let _: dyn Provider;
}

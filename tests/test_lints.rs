#![allow(clippy::mixed_attributes_style)]

use thiserror::Error;

pub use std::error::Error;

#[test]
fn test_unused_qualifications() {
    #![deny(unused_qualifications)]

    // Expansion of derive(Error) macro can't know whether something like
    // std::error::Error is already imported in the caller's scope so it must
    // suppress unused_qualifications.

    #[derive(Debug, Error)]
    #[error("...")]
    pub struct MyError;

    let _: MyError;
}

#[test]
fn test_needless_lifetimes() {
    #![allow(dead_code)]
    #![deny(clippy::needless_lifetimes)]

    #[derive(Debug, Error)]
    #[error("...")]
    pub enum MyError<'a> {
        A(#[from] std::io::Error),
        B(&'a ()),
    }

    let _: MyError;
}

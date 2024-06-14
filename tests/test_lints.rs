#![allow(clippy::mixed_attributes_style)]

use thiserror::Error;

pub use core::error::Error;

#[test]
fn test_unused_qualifications() {
    #![deny(unused_qualifications)]

    // Expansion of derive(Error) macro can't know whether something like
    // core::error::Error is already imported in the caller's scope so it must
    // suppress unused_qualifications.

    #[derive(Debug, Error)]
    #[error("...")]
    pub struct MyError;

    let _: MyError;
}

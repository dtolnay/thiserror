#![cfg_attr(not(feature = "std"), feature(error_in_core))]
#![allow(clippy::mixed_attributes_style)]

use thiserror::Error;

// std or core
pub use thiserror::error::Error;

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

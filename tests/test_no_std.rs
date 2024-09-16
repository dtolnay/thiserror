// There is no negative required-features to suppress this test
// for not(feature = "std").
#![cfg(not(feature = "std"))]
#![no_std]

use core::error::Error as _;
use thiserror::Error;

#[derive(Error, Debug, Default)]
#[error("io")]
pub struct IoError {}

#[derive(Error, Debug)]
#[error("implicit source")]
pub struct ImplicitSource {
    source: IoError,
}

#[derive(Error, Debug)]
#[error("explicit source")]
pub struct ExplicitSource {
    source: i32,
    #[source]
    io: IoError,
}

#[test]
fn test_implicit_source() {
    let io = IoError::default();
    let error = ImplicitSource { source: io };
    error.source().unwrap().downcast_ref::<IoError>().unwrap();
}

#[test]
fn test_explicit_source() {
    let io = IoError::default();
    let error = ExplicitSource { source: 0, io };
    error.source().unwrap().downcast_ref::<IoError>().unwrap();
}

macro_rules! error_from_macro {
    ($($variants:tt)*) => {
        #[derive(Error)]
        #[derive(Debug)]
        pub enum MacroSource {
            $($variants)*
        }
    }
}

// Test that we generate impls with the proper hygiene
#[rustfmt::skip]
error_from_macro! {
    #[error("Something")]
    Variant(#[from] IoError)
}

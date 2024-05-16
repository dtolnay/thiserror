#![cfg_attr(not(feature = "std"), feature(error_in_core))]
#![deny(deprecated, clippy::all, clippy::pedantic)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[deprecated]
    #[error("...")]
    Deprecated,
}

#![deny(deprecated)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[deprecated]
    #[error("...")]
    Deprecated,
}

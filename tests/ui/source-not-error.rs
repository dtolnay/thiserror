use thiserror::Error;

#[derive(Error, Debug)]
#[error("...")]
pub struct Error {
    source: String,
}

fn main() {}

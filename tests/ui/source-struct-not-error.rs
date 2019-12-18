use thiserror::Error;

#[derive(Error, Debug)]
#[error("...")]
pub struct ErrorStruct {
    source: String,
}

fn main() {}

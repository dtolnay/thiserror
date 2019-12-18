use thiserror::Error;

#[derive(Error, Debug)]
#[error("...")]
pub enum ErrorEnum {
    Broken {
        source: String,
    },
}

fn main() {}

use thiserror::Error;

#[derive(Error, Debug)]
#[error("...")]
#[boxing]
pub struct AlsoMissing {
    #[from] err: std::io::Error,
}

fn main() {}

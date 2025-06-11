use thiserror::Error;

#[derive(Error, Debug)]
pub enum Missing {
    #[error(transparent)]
    NotThere(#[boxing] std::io::Error),
}

#[derive(Error, Debug)]
#[error("...")]
pub struct AlsoMissing {
    #[boxing] err: std::io::Error,
}

fn main() {}

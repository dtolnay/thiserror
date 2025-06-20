use thiserror::Error;

#[derive(Error, Debug)]
pub enum Dups {
    #[error(transparent)]
    Twice(#[boxing] #[boxing] #[from] std::io::Error),
}

fn main() {}

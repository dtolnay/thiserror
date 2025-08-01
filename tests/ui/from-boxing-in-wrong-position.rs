use thiserror::Error;

#[derive(Debug, Error)]
#[from(boxing)]
enum E {
    #[error("...")]
    A,
}

fn main() {}

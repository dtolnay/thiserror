use thiserror::Error;

#[derive(Debug, Error)]
pub struct S {
    #[from(boxing)]
    a: std::io::Error,

    #[from(boxing)]
    b: std::io::Error,
}

fn main() {}

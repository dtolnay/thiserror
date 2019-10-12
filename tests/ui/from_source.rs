use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
struct BracedWithSourceDuplicate {
    source: io::Error,
// this should cause compilation error
    #[source]
    cause: io::Error,
}

#[derive(Error, Debug)]
struct BracedWithFromDuplicate1 {
    #[from]
    source: io::Error,
// this should cause compilation error
    #[from]
    cause: io::Error,
}

#[derive(Error, Debug)]
struct BracedWithFromWithoutSource {
    source: io::Error,
// this should cause compilation error
    #[from]
    cause: io::Error,
}

fn main() {}

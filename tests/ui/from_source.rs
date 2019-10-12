use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
struct BracedWithSourceDuplicate {
    source: io::Error,
// this should cause compilation error
    #[source]
    cause: io::Error,
}

fn main() {}

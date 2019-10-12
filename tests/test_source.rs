use std::error::Error as _;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("implicit source")]
pub struct ImplicitSource {
    source: io::Error,
}

#[derive(Error, Debug)]
#[error("explicit source")]
pub struct ExplicitSource {
    source: String,
    #[source]
    io: io::Error,
}

#[test]
fn test_implicit_source() {
    let io = io::Error::new(io::ErrorKind::Other, "oh no!");
    let error = ImplicitSource { source: io };
    error.source().unwrap().downcast_ref::<io::Error>().unwrap();
}

#[test]
fn test_explicit_source() {
    let io = io::Error::new(io::ErrorKind::Other, "oh no!");
    let error = ExplicitSource {
        source: String::new(),
        io,
    };
    error.source().unwrap().downcast_ref::<io::Error>().unwrap();
}

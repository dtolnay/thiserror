use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("braced error: {}", msg)]
struct BracedError {
    msg: String,
}

#[derive(Error, Debug)]
#[error("braced error")]
struct BracedUnused {
    extra: usize,
}

#[derive(Error, Debug)]
#[error("tuple error: {}", .0)]
struct TupleError(usize);

#[derive(Error, Debug)]
#[error("unit error")]
struct UnitError;

fn assert<T: Display>(expected: &str, value: T) {
    assert_eq!(expected, value.to_string());
}

#[test]
fn test_display() {
    assert(
        "braced error: T",
        BracedError {
            msg: "T".to_owned(),
        },
    );
    assert("braced error", BracedUnused { extra: 0 });
    assert("tuple error: 0", TupleError(0));
    assert("unit error", UnitError);
}

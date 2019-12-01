use std::fmt::Display;
use thiserror::Error;

fn assert<T: Display>(expected: &str, value: T) {
    assert_eq!(expected, value.to_string());
}

#[test]
fn test_braced() {
    #[derive(Error, Debug)]
    #[error("braced error: {msg}")]
    struct Error {
        msg: String,
    }

    let msg = "T".to_owned();
    assert("braced error: T", Error { msg });
}

#[test]
fn test_braced_unused() {
    #[derive(Error, Debug)]
    #[error("braced error")]
    struct Error {
        extra: usize,
    }

    assert("braced error", Error { extra: 0 });
}

#[test]
fn test_tuple() {
    #[derive(Error, Debug)]
    #[error("tuple error: {0}")]
    struct Error(usize);

    assert("tuple error: 0", Error(0));
}

#[test]
fn test_unit() {
    #[derive(Error, Debug)]
    #[error("unit error")]
    struct Error;

    assert("unit error", Error);
}

#[test]
fn test_enum() {
    #[derive(Error, Debug)]
    enum Error {
        #[error("braced error: {id}")]
        Braced { id: usize },
        #[error("tuple error: {0}")]
        Tuple(usize),
        #[error("unit error")]
        Unit,
    }

    assert("braced error: 0", Error::Braced { id: 0 });
    assert("tuple error: 0", Error::Tuple(0));
    assert("unit error", Error::Unit);
}

#[test]
fn test_constants() {
    #[derive(Error, Debug)]
    #[error("{MSG}: {id:?} (code {CODE:?})")]
    struct Error {
        id: &'static str,
    }

    const MSG: &str = "failed to do";
    const CODE: usize = 9;

    assert("failed to do: \"\" (code 9)", Error { id: "" });
}

#[test]
fn test_inherit() {
    #[derive(Error, Debug)]
    #[error("{0}")]
    enum Error {
        Some(&'static str),
        #[error("other error")]
        Other(&'static str),
    }

    assert("some error", Error::Some("some error"));
    assert("other error", Error::Other("..."));
}

#[test]
fn test_brace_escape() {
    #[derive(Error, Debug)]
    #[error("fn main() {{}}")]
    struct Error;

    assert("fn main() {}", Error);
}

#[test]
fn test_expr() {
    #[derive(Error, Debug)]
    #[error("1 + 1 = {}", 1 + 1)]
    struct Error;
    assert("1 + 1 = 2", Error);
}

#[test]
fn test_nested() {
    #[derive(Error, Debug)]
    #[error("!bool = {}", not(.0))]
    struct Error(bool);

    fn not(bool: &bool) -> bool {
        !*bool
    }

    assert("!bool = false", Error(true));
}

#[test]
fn test_void() {
    #[derive(Error, Debug)]
    #[error("...")]
    pub enum Error {}
}

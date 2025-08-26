#![feature(error_generic_member_access)]

use backtrace::Backtrace;
use crossterm::style::Stylize;
use thiserror::{thiserror, Error};

#[thiserror]
struct EmptyError;

#[thiserror]
struct UpperError {
    #[from]
    src: EmptyError,
}

#[thiserror]
struct UnhandledException {
    code: u32,
    more_code: u64,
}

#[thiserror]
enum Errors {
    #[error("{:?}", 0)]
    Code(u32),
    #[error("{0}")]
    Code1(u32),
    Unit,
    Code2(u64),
    Struct {
        code: u32,
    },
}

#[derive(Error, Debug)]
enum Errors1 {
    #[error("{:?}", self)]
    Code(u32, u64),
}

fn main() -> Result<(), EmptyError> {
    small_function()?;

    Ok(())
}

fn small_function() -> Result<(), EmptyError> {
    let err = EmptyError {
        backtrace: Backtrace::new(),
    };
    let res: Result<(), EmptyError> = Result::Err(err);
    res?;

    Ok(())
}

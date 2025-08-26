#![feature(error_generic_member_access)]

use backtrace::Backtrace;
use crossterm::style::Stylize;
use thiserror::{capture, thiserror, with_backtrace, Error};

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
pub enum Errors {
    #[error("{:?}", 0)]
    Code(u32),
    #[error("{0}")]
    Code1(u32),
    Unit,
    Code2(u64),
    Struct {
        code: u32,
    },
    StructFrom {
        #[from]
        source: UnhandledException,
    },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
enum Errors1 {
    #[error("{:?}", self)]
    Code(u32, u64),
}

#[thiserror]
struct WrapAnyhow(#[from] anyhow::Error);

fn main() -> Result<(), EmptyError> {
    small_function()?;

    Ok(())
}

fn small_function() -> Result<(), EmptyError> {
    let err = with_backtrace!(EmptyError {});
    let err = EmptyError {
        backtrace: capture!(),
    };
    let res: Result<(), EmptyError> = Result::Err(err);
    res?;

    Ok(())
}

fn parent() -> Result<(), Errors> {
    f1()?;
    anyhow_function()?;

    Ok(())
}

fn f1() -> Result<(), UnhandledException> {
    Err(with_backtrace!(UnhandledException {
        code: 1,
        more_code: 2
    }))
}

fn anyhow_function() -> Result<(), anyhow::Error> {
    Err(anyhow::anyhow!("This is an anyhow error"))
}

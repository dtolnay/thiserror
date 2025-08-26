#![feature(error_generic_member_access)]

use backtrace::Backtrace;
use crossterm::style::Stylize;
use thiserror::{Error, thiserror};

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
    more_code: u64
}

fn main() -> Result<(), EmptyError> {
    let err = EmptyError {
        backtrace: Backtrace::new()
    };
    let res: Result<(), EmptyError> = Result::Err(err);
    res?;

    Ok(())
}

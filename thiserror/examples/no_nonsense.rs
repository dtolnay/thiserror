use backtrace::{Backtrace, BacktraceFmt};

use crossterm::style::Stylize;
use thiserror::{Error, thiserror};

#[thiserror]
struct EmptyError;

fn main() -> Result<(), EmptyError> {

    let err = EmptyError {
        backtrace: Backtrace::new(),
    };
    let res: Result<(), EmptyError> = Result::Err(err);
    res?;

    Ok(())
}

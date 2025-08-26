use backtrace::{Backtrace, BacktraceFmt};

use thiserror::{thiserror, Error};

#[thiserror]
struct EmptyError;

#[test]
fn test_empty() -> Result<(), EmptyError> {
    color_backtrace::install();

    let err = EmptyError {
        backtrace: Backtrace::new(),
    };
    let res = Result::Err(err);
    res?;

    Ok(())
}

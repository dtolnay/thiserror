use backtrace::{Backtrace, BacktraceFmt};

use crossterm::style::Stylize;
use thiserror::{Error, pretty_bt::PrettyBacktrace, thiserror};

#[thiserror]
struct EmptyError;

fn main() -> Result<(), EmptyError> {
    // This handles panics
    color_backtrace::install();

    dbg!(
        "Red foreground color & blue background.".red().on_blue()
    );
    let err = EmptyError {
        backtrace: PrettyBacktrace::new(),
    };
    let res: Result<(), EmptyError> = Result::Err(err);
    // res.unwrap();
    res?;

    Ok(())
}

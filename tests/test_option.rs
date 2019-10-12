#![cfg(thiserror_nightly_testing)]
#![feature(backtrace)]

use std::backtrace::Backtrace;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("...")]
pub struct OptSourceNoBacktraceStruct {
    #[source]
    source: Option<anyhow::Error>,
}

#[derive(Error, Debug)]
#[error("...")]
pub struct OptSourceAlwaysBacktraceStruct {
    #[source]
    source: Option<anyhow::Error>,
    backtrace: Backtrace,
}

#[derive(Error, Debug)]
pub enum OptSourceNoBacktraceEnum {
    #[error("...")]
    Test {
        #[source]
        source: Option<anyhow::Error>,
    },
}

#[derive(Error, Debug)]
pub enum OptSourceAlwaysBacktraceEnum {
    #[error("...")]
    Test {
        #[source]
        source: Option<anyhow::Error>,
        backtrace: Backtrace,
    },
}

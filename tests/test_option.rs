#![cfg(thiserror_nightly_testing)]
#![feature(backtrace)]

use std::backtrace::Backtrace;
use thiserror::Error;

pub mod structs {
    use super::*;

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct OptSourceNoBacktrace {
        #[source]
        source: Option<anyhow::Error>,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct OptSourceAlwaysBacktrace {
        #[source]
        source: Option<anyhow::Error>,
        backtrace: Backtrace,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct NoSourceOptBacktrace {
        #[backtrace]
        backtrace: Option<Backtrace>,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct AlwaysSourceOptBacktrace {
        source: anyhow::Error,
        #[backtrace]
        backtrace: Option<Backtrace>,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct OptSourceOptBacktrace {
        #[source]
        source: Option<anyhow::Error>,
        #[backtrace]
        backtrace: Option<Backtrace>,
    }
}

pub mod enums {
    use super::*;

    #[derive(Error, Debug)]
    pub enum OptSourceNoBacktrace {
        #[error("...")]
        Test {
            #[source]
            source: Option<anyhow::Error>,
        },
    }

    #[derive(Error, Debug)]
    pub enum OptSourceAlwaysBacktrace {
        #[error("...")]
        Test {
            #[source]
            source: Option<anyhow::Error>,
            backtrace: Backtrace,
        },
    }

    #[derive(Error, Debug)]
    pub enum NoSourceOptBacktrace {
        #[error("...")]
        Test {
            #[backtrace]
            backtrace: Option<Backtrace>,
        },
    }

    #[derive(Error, Debug)]
    pub enum AlwaysSourceOptBacktrace {
        #[error("...")]
        Test {
            source: anyhow::Error,
            #[backtrace]
            backtrace: Option<Backtrace>,
        },
    }

    #[derive(Error, Debug)]
    pub enum OptSourceOptBacktrace {
        #[error("...")]
        Test {
            #[source]
            source: Option<anyhow::Error>,
            #[backtrace]
            backtrace: Option<Backtrace>,
        },
    }
}

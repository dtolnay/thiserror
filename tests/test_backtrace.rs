#![cfg_attr(feature = "stdbacktrace", feature(backtrace))]

use thiserror::Error;

#[derive(Error, Debug)]
#[error("...")]
pub struct Inner;

#[cfg(feature = "stdbacktrace")]
fn backtrace() -> std::backtrace::Backtrace {
    std::backtrace::Backtrace::capture()
}

#[cfg(not(feature = "stdbacktrace"))]
fn backtrace() -> backtrace::Backtrace {
    backtrace::Backtrace::new()
}

pub mod structs {
    use super::{backtrace, Inner};
    #[cfg(not(feature = "stdbacktrace"))]
    use backtrace::Backtrace;
    #[cfg(feature = "stdbacktrace")]
    use std::backtrace::Backtrace;
    #[cfg(feature = "stdbacktrace")]
    use std::error::Error;
    use std::sync::Arc;
    use thiserror::Error;

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct PlainBacktrace {
        backtrace: Backtrace,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct ExplicitBacktrace {
        #[backtrace]
        backtrace: Backtrace,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct OptBacktrace {
        #[backtrace]
        backtrace: Option<Backtrace>,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct ArcBacktrace {
        #[backtrace]
        backtrace: Arc<Backtrace>,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct BacktraceFrom {
        #[from]
        source: Inner,
        #[backtrace]
        backtrace: Backtrace,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct OptBacktraceFrom {
        #[from]
        source: Inner,
        #[backtrace]
        backtrace: Option<Backtrace>,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct ArcBacktraceFrom {
        #[from]
        source: Inner,
        #[backtrace]
        backtrace: Arc<Backtrace>,
    }

    #[test]
    #[cfg_attr(not(feature = "stdbacktrace"), allow(unstable_name_collisions))]
    fn test_backtrace() {
        #[cfg(not(feature = "stdbacktrace"))]
        use thiserror::Backtrace as _;

        let error = PlainBacktrace {
            backtrace: backtrace(),
        };
        assert!(error.backtrace().is_some());

        let error = ExplicitBacktrace {
            backtrace: backtrace(),
        };
        assert!(error.backtrace().is_some());

        let error = OptBacktrace {
            backtrace: Some(backtrace()),
        };
        assert!(error.backtrace().is_some());

        let error = ArcBacktrace {
            backtrace: Arc::new(backtrace()),
        };
        assert!(error.backtrace().is_some());

        let error = BacktraceFrom::from(Inner);
        assert!(error.backtrace().is_some());

        let error = OptBacktraceFrom::from(Inner);
        assert!(error.backtrace().is_some());

        let error = ArcBacktraceFrom::from(Inner);
        assert!(error.backtrace().is_some());
    }
}

pub mod enums {
    use super::{backtrace, Inner};
    #[cfg(not(feature = "stdbacktrace"))]
    use backtrace::Backtrace;
    #[cfg(feature = "stdbacktrace")]
    use std::backtrace::Backtrace;
    #[cfg(feature = "stdbacktrace")]
    use std::error::Error;
    use std::sync::Arc;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum PlainBacktrace {
        #[error("...")]
        Test { backtrace: Backtrace },
    }

    #[derive(Error, Debug)]
    pub enum ExplicitBacktrace {
        #[error("...")]
        Test {
            #[backtrace]
            backtrace: Backtrace,
        },
    }

    #[derive(Error, Debug)]
    pub enum OptBacktrace {
        #[error("...")]
        Test {
            #[backtrace]
            backtrace: Option<Backtrace>,
        },
    }

    #[derive(Error, Debug)]
    pub enum ArcBacktrace {
        #[error("...")]
        Test {
            #[backtrace]
            backtrace: Arc<Backtrace>,
        },
    }

    #[derive(Error, Debug)]
    pub enum BacktraceFrom {
        #[error("...")]
        Test {
            #[from]
            source: Inner,
            #[backtrace]
            backtrace: Backtrace,
        },
    }

    #[derive(Error, Debug)]
    pub enum OptBacktraceFrom {
        #[error("...")]
        Test {
            #[from]
            source: Inner,
            #[backtrace]
            backtrace: Option<Backtrace>,
        },
    }

    #[derive(Error, Debug)]
    pub enum ArcBacktraceFrom {
        #[error("...")]
        Test {
            #[from]
            source: Inner,
            #[backtrace]
            backtrace: Arc<Backtrace>,
        },
    }

    #[test]
    #[cfg_attr(not(feature = "stdbacktrace"), allow(unstable_name_collisions))]
    fn test_backtrace() {
        #[cfg(not(feature = "stdbacktrace"))]
        use thiserror::Backtrace as _;

        let error = PlainBacktrace::Test {
            backtrace: backtrace(),
        };
        assert!(error.backtrace().is_some());

        let error = ExplicitBacktrace::Test {
            backtrace: backtrace(),
        };
        assert!(error.backtrace().is_some());

        let error = OptBacktrace::Test {
            backtrace: Some(backtrace()),
        };
        assert!(error.backtrace().is_some());

        let error = ArcBacktrace::Test {
            backtrace: Arc::new(backtrace()),
        };
        assert!(error.backtrace().is_some());

        let error = BacktraceFrom::from(Inner);
        assert!(error.backtrace().is_some());

        let error = OptBacktraceFrom::from(Inner);
        assert!(error.backtrace().is_some());

        let error = ArcBacktraceFrom::from(Inner);
        assert!(error.backtrace().is_some());
    }
}

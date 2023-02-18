#![cfg_attr(
    provide_any,
    feature(error_generic_member_access, provide_any)
)]

use thiserror::Error;

#[derive(Error, Debug)]
#[error("...")]
pub struct Inner;

#[cfg(has_backtrace)]
#[derive(Error, Debug)]
#[error("...")]
pub struct InnerBacktrace {
    backtrace: std::backtrace::Backtrace,
}

#[cfg(has_backtrace)]
pub mod structs {
    use super::{Inner, InnerBacktrace};
    #[cfg(thiserror_nightly_testing)]
    use std::any;
    use std::backtrace::Backtrace;
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
    pub struct CombinedBacktraceFrom {
        #[from]
        #[backtrace]
        source: InnerBacktrace,
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

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct AnyhowBacktrace {
        #[backtrace]
        source: anyhow::Error,
    }

    #[derive(Error, Debug)]
    #[error("...")]
    pub struct BoxDynErrorBacktrace {
        #[backtrace]
        source: Box<dyn Error>,
    }

    #[cfg(thiserror_nightly_testing)]
    #[test]
    fn test_backtrace() {
        let error = PlainBacktrace {
            backtrace: Backtrace::capture(),
        };
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = ExplicitBacktrace {
            backtrace: Backtrace::capture(),
        };
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = OptBacktrace {
            backtrace: Some(Backtrace::capture()),
        };
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = ArcBacktrace {
            backtrace: Arc::new(Backtrace::capture()),
        };
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = BacktraceFrom::from(Inner);
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = CombinedBacktraceFrom::from(InnerBacktrace {
            backtrace: Backtrace::capture(),
        });
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = OptBacktraceFrom::from(Inner);
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = ArcBacktraceFrom::from(Inner);
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = AnyhowBacktrace {
            source: anyhow::Error::msg("..."),
        };
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = BoxDynErrorBacktrace {
            source: Box::new(PlainBacktrace {
                backtrace: Backtrace::capture(),
            }),
        };
        assert!(any::request_ref::<Backtrace>(&error).is_some());
    }

    #[cfg(not(thiserror_nightly_testing))]
    #[allow(unused_must_use)]
    #[test]
    // Make sure we can compile a derived Error for structures containing backtraces
    fn test_backtrace() {
        PlainBacktrace {
            backtrace: Backtrace::capture(),
        };

        ExplicitBacktrace {
            backtrace: Backtrace::capture(),
        };

        OptBacktrace {
            backtrace: Some(Backtrace::capture()),
        };

        ArcBacktrace {
            backtrace: Arc::new(Backtrace::capture()),
        };

        BacktraceFrom::from(Inner);

        CombinedBacktraceFrom::from(InnerBacktrace {
            backtrace: Backtrace::capture(),
        });

        let error = OptBacktraceFrom::from(Inner);
        assert!(error.backtrace.is_some());

        ArcBacktraceFrom::from(Inner);

        AnyhowBacktrace {
            source: anyhow::Error::msg("..."),
        };

        BoxDynErrorBacktrace {
            source: Box::new(PlainBacktrace {
                backtrace: Backtrace::capture(),
            }),
        };
    }

    #[cfg(thiserror_nightly_testing)]
    // https://github.com/dtolnay/thiserror/issues/185 -- std::error::Error and
    // std::any::Provide both have a method called 'provide', so directly
    // calling it from generated code could be ambiguous.
    #[test]
    fn test_provide_name_collision() {
        use std::any::Provider;

        #[derive(Error, Debug)]
        #[error("...")]
        struct MyError {
            #[source]
            #[backtrace]
            x: std::io::Error,
        }

        let _: dyn Error;
        let _: dyn Provider;
    }
}

#[cfg(has_backtrace)]
pub mod enums {
    use super::{Inner, InnerBacktrace};
    #[cfg(thiserror_nightly_testing)]
    use std::any;
    use std::backtrace::Backtrace;
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
    pub enum CombinedBacktraceFrom {
        #[error("...")]
        Test {
            #[from]
            #[backtrace]
            source: InnerBacktrace,
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

    #[cfg(thiserror_nightly_testing)]
    #[test]
    fn test_backtrace() {
        let error = PlainBacktrace::Test {
            backtrace: Backtrace::capture(),
        };
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = ExplicitBacktrace::Test {
            backtrace: Backtrace::capture(),
        };
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = OptBacktrace::Test {
            backtrace: Some(Backtrace::capture()),
        };
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = ArcBacktrace::Test {
            backtrace: Arc::new(Backtrace::capture()),
        };
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = BacktraceFrom::from(Inner);
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = CombinedBacktraceFrom::from(InnerBacktrace {
            backtrace: Backtrace::capture(),
        });
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = OptBacktraceFrom::from(Inner);
        assert!(any::request_ref::<Backtrace>(&error).is_some());

        let error = ArcBacktraceFrom::from(Inner);
        assert!(any::request_ref::<Backtrace>(&error).is_some());
    }

    #[cfg(not(thiserror_nightly_testing))]
    #[allow(unused_must_use)]
    #[test]
    // Make sure we can compile a derived Error for enums containing backtraces
    fn test_backtrace() {
        PlainBacktrace::Test {
            backtrace: Backtrace::capture(),
        };

        ExplicitBacktrace::Test {
            backtrace: Backtrace::capture(),
        };

        OptBacktrace::Test {
            backtrace: Some(Backtrace::capture()),
        };

        ArcBacktrace::Test {
            backtrace: Arc::new(Backtrace::capture()),
        };

        BacktraceFrom::from(Inner);

        CombinedBacktraceFrom::from(InnerBacktrace {
            backtrace: Backtrace::capture(),
        });

        let error = OptBacktraceFrom::from(Inner);
        assert!({
            let OptBacktraceFrom::Test {
                backtrace,
                source: _,
            } = error;
            backtrace.is_some()
        });

        ArcBacktraceFrom::from(Inner);
    }
}

#[test]
#[cfg_attr(not(has_backtrace), ignore)]
fn test_backtrace() {}

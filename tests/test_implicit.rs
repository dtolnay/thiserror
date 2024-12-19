use std::{backtrace, panic::Location, sync::Arc};

use thiserror::{Error, ImplicitField};

#[derive(Error, Debug)]
#[error("Inner")]
pub struct Inner;

#[derive(Debug)]
pub struct ImplicitBacktrace(pub backtrace::Backtrace);

impl ImplicitField for ImplicitBacktrace {
    fn generate() -> Self {
        Self(backtrace::Backtrace::force_capture())
    }
}

#[derive(Debug)]
pub struct ImplicitSimple;

impl ImplicitField for ImplicitSimple {
    fn generate() -> Self {
        Self
    }
}

#[derive(Error, Debug)]
#[error("location: {location:?}")]
pub struct ErrorStruct {
    #[from]
    source: Inner,
    #[implicit]
    backtrace: ImplicitBacktrace,
    #[implicit]
    location: &'static Location<'static>,
    #[implicit]
    simple_arc: Arc<ImplicitSimple>,
    #[implicit]
    simple_opt: Option<ImplicitSimple>,
}

#[derive(Error, Debug)]
#[error("location: {location:?}")]
pub enum ErrorEnum {
    #[error("location: {location:?}")]
    Test {
        #[from]
        source: Inner,
        #[implicit]
        backtrace: ImplicitBacktrace,
        #[implicit]
        location: &'static Location<'static>,
        #[implicit]
        simple_arc: Arc<ImplicitSimple>,
        #[implicit]
        simple_opt: Option<ImplicitSimple>,
    },
}

#[test]
fn test_implicit() {
    let base_location = Location::caller();
    let assert_location = |location: &'static Location<'static>| {
        assert_eq!(location.file(), file!(), "location: {location:?}");
        assert!(
            location.line() > base_location.line(),
            "location: {location:?}"
        );
    };

    let error = ErrorStruct::from(Inner);
    assert_location(error.location);
    assert_eq!(
        error.backtrace.0.status(),
        backtrace::BacktraceStatus::Captured
    );

    let ErrorEnum::Test {
        backtrace,
        location,
        ..
    } = ErrorEnum::from(Inner);
    assert_location(location);
    assert_eq!(backtrace.0.status(), backtrace::BacktraceStatus::Captured);
}

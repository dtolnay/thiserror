use std::{backtrace, sync::Arc};

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
pub struct Location(pub &'static core::panic::Location<'static>);

impl Default for Location {
    #[track_caller]
    fn default() -> Self {
        Self(core::panic::Location::caller())
    }
}

impl ImplicitField for Location {
    #[track_caller]
    fn generate() -> Self {
        Self::default()
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
    location: Location,
    #[implicit]
    location_arc: Arc<Location>,
    #[implicit]
    location_opt: Option<Location>,
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
        location: Location,
        #[implicit]
        location_arc: Arc<Location>,
        #[implicit]
        location_opt: Option<Location>,
    },
}

#[test]
fn test_implicit() {
    let base_location = Location::default();
    let assert_location = |location: &Location| {
        assert_eq!(location.0.file(), file!(), "location: {location:?}");
        assert!(
            location.0.line() > base_location.0.line(),
            "location: {location:?}"
        );
    };

    let error = ErrorStruct::from(Inner);
    assert_location(&error.location);
    assert_location(&error.location_arc);
    assert_location(error.location_opt.as_ref().unwrap());
    assert_eq!(
        error.backtrace.0.status(),
        backtrace::BacktraceStatus::Captured
    );

    let ErrorEnum::Test {
        source: _,
        backtrace,
        location,
        location_arc,
        location_opt,
    } = ErrorEnum::from(Inner);
    assert_location(&location);
    assert_location(&location_arc);
    assert_location(location_opt.as_ref().unwrap());
    assert_eq!(backtrace.0.status(), backtrace::BacktraceStatus::Captured);
}

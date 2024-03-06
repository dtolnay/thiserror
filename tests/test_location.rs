use std::{fmt::Debug, io, panic::Location};
use thiserror::Error;

#[derive(Error, Debug)]
enum MError {
    #[error("At {location}: location test error, sourced from {other}")]
    Test {
        #[from]
        other: io::Error,
        location: &'static Location<'static>,
    },
}

#[derive(Error, Debug)]
#[error("At {location} test error, sourced from {other}")]
pub struct TestError {
    #[from]
    other: io::Error,
    location: &'static Location<'static>,
}

#[test]
#[should_panic]
fn test_enum() {
    fn inner() -> Result<(), MError> {
        Err(io::Error::new(io::ErrorKind::AddrInUse, String::new()))?;
        Ok(())
    }

    inner().unwrap();
}

#[test]
#[should_panic]
fn test_struct() {
    fn inner() -> Result<(), TestError> {
        Err(io::Error::new(io::ErrorKind::AddrInUse, String::new()))?;
        Ok(())
    }

    inner().unwrap();
}

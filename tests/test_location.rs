use std::{fmt::Debug, io, panic::Location};
use thiserror::Error;

#[derive(Error, Debug)]
enum MError {
    #[error("At {location}: location test error, sourced from {other}")]
    Test {
        #[location]
        location: &'static Location<'static>,
        #[from]
        other: io::Error,
    },
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

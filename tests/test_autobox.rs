#![cfg(feature = "autobox")]

use thiserror::Error;

#[derive(Debug, Error)]
#[error("not great")]
struct VeryLargeError {
    a: [u8; 2048],
}

impl VeryLargeError {
    fn new() -> Self {
        Self { a: [0; 2048] }
    }
}

#[derive(Debug, Error)]
enum ErrorEnum {
    #[error("bad")]
    Any(#[from] anyhow::Error),

    #[error("worse")]
    Big(#[from] VeryLargeError),
}

/// External code may still return large errors...
fn do_something() -> Result<(), VeryLargeError> {
    Err(VeryLargeError::new())
}

/// But we should be able to box them automatically!
fn do_something_else() -> Result<(), Box<ErrorEnum>> {
    do_something()?;

    Ok(())
}

#[test]
fn autobox() {
    let _ = do_something_else().unwrap_err();
}

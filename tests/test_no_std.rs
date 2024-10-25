#![no_std]
use thiserror::Error;

#[derive(Error, Debug)]
#[error("io")]
pub struct IoError;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("A")]
    A,
    #[error("B {0}")]
    B(#[from] IoError),
}

#[test]
#[cfg(not(feature = "std"))]
fn test_no_std() {
    use core::error::Error as _;

    let error = MyError::from(IoError);
    error.source().unwrap().downcast_ref::<IoError>().unwrap();
}

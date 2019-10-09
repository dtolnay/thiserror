use std::fmt::{self, Display};
use thiserror::Error;

macro_rules! unimplemented_display {
    ($ty:ty) => {
        impl Display for $ty {
            fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
                unimplemented!()
            }
        }
    };
}

#[derive(Error, Debug)]
struct BracedError {
    msg: String,
    pos: usize,
}

#[derive(Error, Debug)]
struct TupleError(String, usize);

#[derive(Error, Debug)]
struct UnitError;

unimplemented_display!(BracedError);
unimplemented_display!(TupleError);
unimplemented_display!(UnitError);

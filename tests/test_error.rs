#![deny(clippy::all, clippy::pedantic)]
#![allow(dead_code)]

use std::fmt::{self, Display};
use std::io;
use thiserror::Error;

macro_rules! unimplemented_display {
    ($($tl:lifetime),*; $tp:tt; $ty:ty) => {
        impl<$($tl),*, $tp> Display for $ty {
            fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
                unimplemented!()
            }
        }
    };
    ($tp:tt; $ty:ty) => {
        impl<$tp> Display for $ty {
            fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
                unimplemented!()
            }
        }
    };
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

#[derive(Error, Debug)]
struct WithSource {
    #[source]
    cause: io::Error,
}

#[derive(Error, Debug)]
struct WithAnyhow {
    #[source]
    cause: anyhow::Error,
}

#[derive(Error, Debug)]
enum EnumError {
    Braced {
        #[source]
        cause: io::Error,
    },
    Tuple(#[source] io::Error),
    Unit,
}

#[derive(Error, Debug)]
#[error(bound = std::fmt::Display + std::error::Error + 'static)]
enum WithGeneric<T> {
    Variant,
    Generic(T),
}

#[derive(Error, Debug)]
#[error(bound = std::fmt::Debug + std::error::Error + 'static)]
enum WithGenericFrom<T> {
    Variant,
    Generic(#[from] T),
}

#[derive(Error, Debug)]
#[error(bound = std::fmt::Display + std::fmt::Debug + std::error::Error + 'static)]
enum WithGenericTransparent<T> {
    #[error("variant")]
    Variant,
    #[error(transparent)]
    Generic(#[from] T),
}

#[derive(Error, Debug)]
#[error(bound = std::error::Error + 'static)]
struct WithGenericStruct<T> {
    #[from]
    inner: T,
}

#[derive(Error, Debug)]
#[error(bound = std::error::Error + 'static)]
struct WithGenericStructRef<'a, T> {
    inner: &'a WithGenericStruct<T>,
}

#[derive(Error, Debug)]
#[error(bound = std::error::Error + 'a)]
struct WithGenericStructRefNonStatic<'a, T> {
    inner: &'a WithGenericStruct<T>,
}

#[derive(Error, Debug)]
#[error(bound = std::fmt::Display + std::error::Error + 'static)]
#[error(transparent)]
struct WithGenericStructTransparent<T> {
    #[from]
    inner: T,
}

unimplemented_display!(BracedError);
unimplemented_display!(TupleError);
unimplemented_display!(UnitError);
unimplemented_display!(WithSource);
unimplemented_display!(WithAnyhow);
unimplemented_display!(EnumError);
unimplemented_display!(T; WithGeneric<T>);
unimplemented_display!(T; WithGenericFrom<T>);
unimplemented_display!(T; WithGenericStruct<T>);
unimplemented_display!('a; T; WithGenericStructRef<'a, T>);
unimplemented_display!('a; T; WithGenericStructRefNonStatic<'a, T>);

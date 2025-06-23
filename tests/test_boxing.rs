use thiserror::Error;

#[derive(Error, Debug)]
#[error("...")]
pub struct LargeError {
    a: [u8; 2048],
}

pub fn direct_return_large() -> Result<(), LargeError> {
    Err(LargeError { a: [0; 2048] })
}

pub fn non_boxed() -> Result<(), Autoboxed> {
    let _ = direct_return_large()?;

    Ok(())
}

#[derive(Error, Debug)]
#[error("...")]
pub enum Autoboxed {
    Large(
        #[from]
        #[boxing]
        LargeError,
    ),
}

pub fn autobox() -> Result<(), Box<Autoboxed>> {
    let _ = direct_return_large()?;

    Ok(())
}

#[derive(Error, Debug)]
#[error("...")]
pub struct Autoboxed2 {
    #[from]
    #[boxing]
    err: LargeError,
}

pub fn autobox2() -> Result<(), Box<Autoboxed2>> {
    let _ = direct_return_large()?;

    Ok(())
}

#[derive(Error, Debug)]
#[error("...")]
pub struct Autoboxed3 {
    #[boxing]
    #[from]
    err: LargeError,
}

pub fn autobox3() -> Result<(), Box<Autoboxed3>> {
    let _ = direct_return_large()?;

    Ok(())
}

#[derive(Error, Debug)]
#[error("...")]
pub enum Multiple {
    #[error(transparent)]
    A(
        #[from]
        #[boxing]
        LargeError,
    ),

    #[error(transparent)]
    B {
        #[from]
        #[boxing]
        named_field: std::io::Error,
    },
}

pub fn std_fallible() -> std::io::Result<()> {
    unimplemented!()
}

pub fn boxes_both() -> Result<(), Box<Multiple>> {
    let _ = direct_return_large()?;
    let _ = std_fallible()?;

    Ok(())
}

/// Deliberatly contrived typed to exercise the lifetimes and generics part of the proc macro.
pub trait Origin<'a>: std::error::Error {
    fn origin(&self) -> &'a str;
}

#[derive(Debug, Error)]
#[error("origin {0}")]
pub struct SomeOrigin<'a>(&'a str);

impl<'a> Origin<'a> for SomeOrigin<'a> {
    fn origin(&self) -> &'a str {
        self.0
    }
}

#[derive(Debug, Error)]
pub enum SomeErr<'a, T>
where
    T: Origin<'a>,
{
    #[error("...")]
    A(
        #[from]
        #[boxing]
        T,
    ),

    #[error("...")]
    B(&'a str),
}

pub fn bad_thing<'a>(input: &'a str) -> Result<(), SomeOrigin<'a>> {
    Err(SomeOrigin(input))
}

pub fn boxing_with_lifetimes_and_generics<'a>(
    input: &'a str,
) -> Result<(), Box<SomeErr<'a, SomeOrigin<'a>>>> {
    let _ = bad_thing(input)?;
    Ok(())
}

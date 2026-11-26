use core::error::Error;
use core::panic::UnwindSafe;

#[doc(hidden)]
pub trait AsDynError<'a>: Sealed {
    fn __thiserror_as_dyn_error(&self) -> &(dyn Error + 'a);
}

impl<'a, T: Error + 'a> AsDynError<'a> for T {
    #[inline]
    fn __thiserror_as_dyn_error(&self) -> &(dyn Error + 'a) {
        self
    }
}

impl<'a> AsDynError<'a> for dyn Error + 'a {
    #[inline]
    fn __thiserror_as_dyn_error(&self) -> &(dyn Error + 'a) {
        self
    }
}

impl<'a> AsDynError<'a> for dyn Error + Send + 'a {
    #[inline]
    fn __thiserror_as_dyn_error(&self) -> &(dyn Error + 'a) {
        self
    }
}

impl<'a> AsDynError<'a> for dyn Error + Send + Sync + 'a {
    #[inline]
    fn __thiserror_as_dyn_error(&self) -> &(dyn Error + 'a) {
        self
    }
}

impl<'a> AsDynError<'a> for dyn Error + Send + Sync + UnwindSafe + 'a {
    #[inline]
    fn __thiserror_as_dyn_error(&self) -> &(dyn Error + 'a) {
        self
    }
}

#[doc(hidden)]
pub trait Sealed {}
impl<T: Error> Sealed for T {}
impl Sealed for dyn Error + '_ {}
impl Sealed for dyn Error + Send + '_ {}
impl Sealed for dyn Error + Send + Sync + '_ {}
impl Sealed for dyn Error + Send + Sync + UnwindSafe + '_ {}

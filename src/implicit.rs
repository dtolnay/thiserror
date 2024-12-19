use core::error::Error;
use core::panic::Location;
#[cfg(feature = "std")]
use std::sync::Arc;

pub trait ImplicitField {
    #[track_caller]
    fn generate() -> Self;

    #[track_caller]
    fn generate_with_source(source: &dyn Error) -> Self
    where
        Self: Sized,
    {
        let _ = source;
        Self::generate()
    }
}

impl ImplicitField for &'static Location<'static> {
    #[track_caller]
    fn generate() -> Self {
        Location::caller()
    }
}

#[cfg(feature = "std")]
impl<T: ImplicitField> ImplicitField for Arc<T> {
    #[track_caller]
    fn generate() -> Self {
        T::generate().into()
    }

    #[track_caller]
    fn generate_with_source(source: &dyn Error) -> Self
    where
        Self: Sized,
    {
        T::generate_with_source(source).into()
    }
}

impl<T: ImplicitField> ImplicitField for Option<T> {
    #[track_caller]
    fn generate() -> Self {
        T::generate().into()
    }

    #[track_caller]
    fn generate_with_source(source: &dyn Error) -> Self
    where
        Self: Sized,
    {
        T::generate_with_source(source).into()
    }
}

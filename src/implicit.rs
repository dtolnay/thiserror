use core::error::Error;
#[cfg(feature = "std")]
use std::sync::Arc;

pub trait ImplicitField {
    // Required method
    #[track_caller]
    fn generate() -> Self;

    // Provided method
    #[track_caller]
    fn generate_with_source(source: &dyn Error) -> Self
    where
        Self: Sized,
    {
        let _ = source;
        Self::generate()
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

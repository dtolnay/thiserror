use std::any::Request;

pub trait ThiserrorProvide: Sealed {
    fn thiserror_provide<'a>(&'a self, request: &mut Request<'a>);
}

impl<T: std::error::Error + ?Sized> ThiserrorProvide for T {
    #[inline]
    fn thiserror_provide<'a>(&'a self, request: &mut Request<'a>) {
        self.provide(request);
    }
}

pub trait Sealed {}
impl<T: ?Sized> Sealed for T {}

use std::error::Error;

pub trait Backtrace: Error {
    fn backtrace(&self) -> Option<&backtrace::Backtrace>;
}

impl Backtrace for &(dyn Backtrace) {
    fn backtrace(&self) -> Option<&backtrace::Backtrace> {
        Backtrace::backtrace(*self)
    }
}

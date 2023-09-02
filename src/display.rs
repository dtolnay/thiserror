use std::fmt::Display;
use std::path::{self, Path, PathBuf};

#[doc(hidden)]
pub trait AsDisplay {
    type Target<'a>: Display
    where
        Self: 'a;

    fn as_display<'a>(&'a self) -> Self::Target<'a>;
}

impl<T> AsDisplay for &T
where
    T: Display,
{
    type Target<'a> = &'a T
    where
        Self: 'a;

    fn as_display<'a>(&'a self) -> Self::Target<'a> {
        *self
    }
}

impl AsDisplay for Path {
    type Target<'a> = path::Display<'a>;

    #[inline]
    fn as_display<'a>(&'a self) -> Self::Target<'a> {
        self.display()
    }
}

impl AsDisplay for PathBuf {
    type Target<'a> = path::Display<'a>;

    #[inline]
    fn as_display<'a>(&'a self) -> Self::Target<'a> {
        self.display()
    }
}

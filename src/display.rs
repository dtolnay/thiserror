use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

#[doc(hidden)]
pub trait AsDisplay<'a> {
    type Target: Display;

    fn as_display(&'a self) -> Self::Target;
}

impl<'a, T> AsDisplay<'a> for &T
where
    T: Display + 'a,
{
    type Target = &'a T;

    fn as_display(&'a self) -> Self::Target {
        *self
    }
}

impl<'a> AsDisplay<'a> for Path {
    type Target = PathDisplay<'a>;

    #[inline]
    fn as_display(&'a self) -> Self::Target {
        PathDisplay(self)
    }
}

impl<'a> AsDisplay<'a> for PathBuf {
    type Target = PathDisplay<'a>;

    #[inline]
    fn as_display(&'a self) -> Self::Target {
        PathDisplay(self.as_path())
    }
}

#[doc(hidden)]
pub struct PathDisplay<'a>(&'a Path);

impl<'a> Display for PathDisplay<'a> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.display().fmt(formatter)
    }
}

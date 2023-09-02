use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

#[doc(hidden)]
pub trait AsDisplay {
    type Target: Display + ?Sized;

    fn as_display(&self) -> &Self::Target;
}

impl<T: Display> AsDisplay for &T {
    type Target = T;

    fn as_display(&self) -> &Self::Target {
        self
    }
}

impl AsDisplay for Path {
    type Target = PathDisplay;

    #[inline]
    fn as_display(&self) -> &Self::Target {
        PathDisplay::new(self)
    }
}

impl AsDisplay for PathBuf {
    type Target = PathDisplay;

    #[inline]
    fn as_display(&self) -> &Self::Target {
        PathDisplay::new(self.as_path())
    }
}

#[doc(hidden)]
#[repr(transparent)]
pub struct PathDisplay(Path);

impl PathDisplay {
    #[inline]
    fn new(path: &Path) -> &Self {
        // SAFETY: PathDisplay is repr(transparent) so casting pointers between
        // it and its payload is safe.
        unsafe { &*(path as *const Path as *const Self) }
    }
}

impl Display for PathDisplay {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.display().fmt(formatter)
    }
}

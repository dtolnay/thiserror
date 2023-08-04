use std::fmt::Display;
use std::path::{Path, PathBuf};

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

    #[inline(always)]
    fn as_display(&self) -> &Self::Target {
        PathDisplay::new(self)
    }
}

impl AsDisplay for PathBuf {
    type Target = PathDisplay;

    #[inline(always)]
    fn as_display(&self) -> &Self::Target {
        PathDisplay::new(self.as_path())
    }
}

#[repr(transparent)]
pub struct PathDisplay(Path);

impl PathDisplay {
    #[inline(always)]
    fn new(path: &Path) -> &Self {
        // SAFETY: PathDisplay is repr(transparent) so casting pointers between
        // it and its payload is safe.
        unsafe { &*(path as *const Path as *const Self) }
    }
}

impl Display for PathDisplay {
    #[inline(always)]
    fn fmt(&self, fmtr: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.display().fmt(fmtr)
    }
}

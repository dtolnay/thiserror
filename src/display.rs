use core::fmt::Display;

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

#[cfg(not(feature = "no_std"))]
mod path {
    use std::path::{Path, PathBuf};

    impl super::AsDisplay for Path {
        type Target = PathDisplay;

        #[inline(always)]
        fn as_display(&self) -> &Self::Target {
            PathDisplay::new(self)
        }
    }

    impl super::AsDisplay for PathBuf {
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
            // SAFETY: PathDisplay is repr(transparent) so casting pointers
            // between it and its payload is safe.
            unsafe { &*(path as *const Path as *const Self) }
        }
    }

    impl core::fmt::Display for PathDisplay {
        #[inline(always)]
        fn fmt(&self, fmtr: &mut core::fmt::Formatter) -> core::fmt::Result {
            self.0.display().fmt(fmtr)
        }
    }
}

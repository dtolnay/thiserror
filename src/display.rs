use core::fmt::Display;
#[cfg(feature = "std")]
use std::path::{self, Path, PathBuf};

pub trait DisplayAsDisplay {
    fn as_display(&self) -> Self;
}

impl<T: Display> DisplayAsDisplay for &T {
    fn as_display(&self) -> Self {
        self
    }
}

#[cfg(feature = "std")]
pub trait PathAsDisplay {
    fn as_display(&self) -> path::Display<'_>;
}

#[cfg(feature = "std")]
impl PathAsDisplay for Path {
    fn as_display(&self) -> path::Display<'_> {
        self.display()
    }
}

#[cfg(feature = "std")]
impl PathAsDisplay for PathBuf {
    fn as_display(&self) -> path::Display<'_> {
        self.display()
    }
}

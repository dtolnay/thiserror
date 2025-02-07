#![no_std]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error::E")]
    E(#[from] SourceError),
}

#[derive(Error, Debug)]
#[error("SourceError {field}")]
pub struct SourceError {
    pub field: i32,
}

#[cfg(test)]
mod tests {
    use crate::{Error, SourceError};
    use core::error::Error as _;
    use core::fmt::{self, Write};
    use core::mem;

    struct Buf<'a>(&'a mut [u8]);

    impl Write for Buf<'_> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            if s.len() <= self.0.len() {
                let (out, rest) = mem::take(&mut self.0).split_at_mut(s.len());
                out.copy_from_slice(s.as_bytes());
                self.0 = rest;
                Ok(())
            } else {
                Err(fmt::Error)
            }
        }
    }

    #[test]
    fn test() {
        let source = SourceError { field: -1 };
        let error = Error::from(source);

        let source = error
            .source()
            .unwrap()
            .downcast_ref::<SourceError>()
            .unwrap();

        let mut msg = [b'~'; 17];
        write!(Buf(&mut msg), "{error}").unwrap();
        assert_eq!(msg, *b"Error::E~~~~~~~~~");

        let mut msg = [b'~'; 17];
        write!(Buf(&mut msg), "{source}").unwrap();
        assert_eq!(msg, *b"SourceError -1~~~");
    }
}

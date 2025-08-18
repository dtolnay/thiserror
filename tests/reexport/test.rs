#[cfg(test)]
mod tests {
    use thiserror_export::Error;

    #[test]
    fn test_this_compiles() {
        #[derive(Error, Debug)]
        #[error(crate = ::thiserror_export)]
        pub enum EnumError {
            #[error("EnumError::E")]
            E(#[from] StructErrorWithDisplay),
        }

        #[derive(Error, Debug)]
        #[error("SourceError {field}")]
        #[error(crate = ::thiserror_export)]
        pub struct StructErrorWithDisplay {
            pub field: i32,
        }

        #[derive(Error, Debug)]
        #[allow(dead_code)]
        #[error(fmt = core::fmt::Octal::fmt)]
        #[error(crate = ::thiserror_export)]
        pub enum EnumErrorWithFmt {
            I16(i16),
            I32 {
                n: i32,
            },
            #[error(fmt = core::fmt::Octal::fmt)]
            I64(i64),
            #[error("...{0}")]
            Other(bool),
        }

        assert!(
            !EnumError::E(StructErrorWithDisplay { field: 1 })
                .to_string()
                .is_empty()
        );
        assert!(!EnumErrorWithFmt::I16(1i16).to_string().is_empty());
    }
}

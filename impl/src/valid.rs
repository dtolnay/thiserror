use crate::ast::{Enum, Input, Struct};
use syn::{Error, Result};

pub(crate) const CHECKED: &str = "checked in validation";

impl Input<'_> {
    pub(crate) fn validate(&self) -> Result<()> {
        match self {
            Input::Struct(input) => input.validate(),
            Input::Enum(input) => input.validate(),
        }
    }
}

impl Struct<'_> {
    fn validate(&self) -> Result<()> {
        // nothing for now
        Ok(())
    }
}

impl Enum<'_> {
    fn validate(&self) -> Result<()> {
        if self.has_display() {
            for variant in &self.variants {
                if variant.attrs.display.is_none() {
                    return Err(Error::new_spanned(
                        variant.original,
                        "missing #[error(\"...\")] display attribute",
                    ));
                }
            }
        }
        Ok(())
    }
}

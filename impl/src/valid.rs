use crate::ast::{Enum, Field, Input, Struct, Variant};
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
        find_duplicate_source(&self.fields)?;
        Ok(())
    }
}

impl Enum<'_> {
    fn validate(&self) -> Result<()> {
        let has_display = self.has_display();
        for variant in &self.variants {
            variant.validate()?;
            if has_display && variant.attrs.display.is_none() {
                return Err(Error::new_spanned(
                    variant.original,
                    "missing #[error(\"...\")] display attribute",
                ));
            }
        }
        Ok(())
    }
}

impl Variant<'_> {
    fn validate(&self) -> Result<()> {
        find_duplicate_source(&self.fields)?;
        Ok(())
    }
}

fn find_duplicate_source(fields: &[Field]) -> Result<()> {
    let mut has_source = false;
    for field in fields {
        if let Some(source) = &field.attrs.source {
            if has_source {
                return Err(Error::new_spanned(
                    source.original,
                    "duplicate #[source] attribute",
                ));
            }
            has_source = true;
        }
    }
    Ok(())
}

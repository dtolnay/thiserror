use crate::ast::{Enum, Field, Input, Struct, Variant};
use crate::attr::Attrs;
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
        check_no_source(&self.attrs)?;
        find_duplicate_source(&self.fields)?;
        Ok(())
    }
}

impl Enum<'_> {
    fn validate(&self) -> Result<()> {
        check_no_source(&self.attrs)?;
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
        check_no_source(&self.attrs)?;
        find_duplicate_source(&self.fields)?;
        Ok(())
    }
}

fn check_no_source(attrs: &Attrs) -> Result<()> {
    if let Some(source) = &attrs.source {
        return Err(Error::new_spanned(
            source.original,
            "not expected here; the #[source] attribute belongs on a specific field",
        ));
    }
    Ok(())
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

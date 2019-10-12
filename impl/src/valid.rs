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
        check_no_source_or_backtrace(&self.attrs)?;
        check_no_duplicate_source_or_backtrace(&self.fields)?;
        for field in &self.fields {
            field.validate()?;
        }
        Ok(())
    }
}

impl Enum<'_> {
    fn validate(&self) -> Result<()> {
        check_no_source_or_backtrace(&self.attrs)?;
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
        check_no_source_or_backtrace(&self.attrs)?;
        check_no_duplicate_source_or_backtrace(&self.fields)?;
        for field in &self.fields {
            field.validate()?;
        }
        Ok(())
    }
}

impl Field<'_> {
    fn validate(&self) -> Result<()> {
        if let Some(display) = &self.attrs.display {
            return Err(Error::new_spanned(
                display.original,
                "not expected here; the #[error(...)] attribute belongs on top of a struct or an enum variant",
            ));
        }
        Ok(())
    }
}

fn check_no_source_or_backtrace(attrs: &Attrs) -> Result<()> {
    if let Some(source) = &attrs.source {
        return Err(Error::new_spanned(
            source.original,
            "not expected here; the #[source] attribute belongs on a specific field",
        ));
    }
    if let Some(backtrace) = &attrs.backtrace {
        return Err(Error::new_spanned(
            backtrace.original,
            "not expected here; the #[backtrace] attribute belongs on a specific field",
        ));
    }
    Ok(())
}

fn check_no_duplicate_source_or_backtrace(fields: &[Field]) -> Result<()> {
    let mut has_source = false;
    let mut has_backtrace = false;
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
        if let Some(backtrace) = &field.attrs.backtrace {
            if has_backtrace {
                return Err(Error::new_spanned(
                    backtrace.original,
                    "duplicate #[backtrace] attribute",
                ));
            }
            has_backtrace = true;
        }
    }
    Ok(())
}

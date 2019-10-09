use syn::parse::Nothing;
use syn::{Field, Result};

pub fn is_source(field: &Field) -> Result<bool> {
    for attr in &field.attrs {
        if attr.path.is_ident("source") {
            syn::parse2::<Nothing>(attr.tokens.clone())?;
            return Ok(true);
        }
    }
    Ok(false)
}

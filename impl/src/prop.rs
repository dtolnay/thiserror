use crate::ast::Enum;

impl Enum<'_> {
    pub(crate) fn has_display(&self) -> bool {
        self.variants
            .iter()
            .any(|variant| variant.attrs.display.is_some())
    }
}

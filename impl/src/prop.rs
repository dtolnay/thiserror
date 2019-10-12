use crate::ast::{Enum, Field, Struct, Variant};
use syn::{Member, Type};

impl Struct<'_> {
    pub(crate) fn source_member(&self) -> Option<&Member> {
        source_member(&self.fields)
    }

    pub(crate) fn backtrace_member(&self) -> Option<&Member> {
        backtrace_member(&self.fields)
    }
}

impl Enum<'_> {
    pub(crate) fn has_source(&self) -> bool {
        self.variants
            .iter()
            .any(|variant| variant.source_member().is_some())
    }

    pub(crate) fn has_backtrace(&self) -> bool {
        self.variants
            .iter()
            .any(|variant| variant.backtrace_member().is_some())
    }

    pub(crate) fn has_display(&self) -> bool {
        self.variants
            .iter()
            .any(|variant| variant.attrs.display.is_some())
    }
}

impl Variant<'_> {
    pub(crate) fn source_member(&self) -> Option<&Member> {
        source_member(&self.fields)
    }

    pub(crate) fn backtrace_member(&self) -> Option<&Member> {
        backtrace_member(&self.fields)
    }
}

impl Field<'_> {
    fn is_source(&self) -> bool {
        self.attrs.source
    }

    fn is_backtrace(&self) -> bool {
        type_is_backtrace(self.ty)
    }
}

fn source_member<'a>(fields: &'a [Field]) -> Option<&'a Member> {
    fields
        .iter()
        .find(|field| field.is_source())
        .map(|field| &field.member)
}

fn backtrace_member<'a>(fields: &'a [Field]) -> Option<&'a Member> {
    fields
        .iter()
        .find(|field| field.is_backtrace())
        .map(|field| &field.member)
}

fn type_is_backtrace(ty: &Type) -> bool {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return false,
    };

    let last = path.segments.last().unwrap();
    last.ident == "Backtrace" && last.arguments.is_empty()
}

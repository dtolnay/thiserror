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
        self.attrs.display.is_some()
            || self
                .variants
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
    fn is_backtrace(&self) -> bool {
        type_is_backtrace(self.ty)
    }
}

fn source_member<'a>(fields: &'a [Field]) -> Option<&'a Member> {
    for field in fields {
        if field.attrs.source.is_some() {
            return Some(&field.member);
        }
    }
    for field in fields {
        match &field.member {
            Member::Named(ident) if ident == "source" => return Some(&field.member),
            _ => {}
        }
    }
    None
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

use crate::ast::{Enum, Field, Struct, Variant};
use crate::span::MemberSpan;
use proc_macro2::Span;
use syn::{Member, Type};

impl Struct<'_> {
    pub(crate) fn from_field(&self) -> Option<&Field> {
        from_field(&self.fields)
    }

    pub(crate) fn source_field(&self) -> Option<&Field> {
        source_field(&self.fields)
    }

    pub(crate) fn backtrace_field(&self) -> Option<&Field> {
        backtrace_field(&self.fields)
    }

    pub(crate) fn distinct_backtrace_field(&self) -> Option<&Field> {
        let backtrace_field = self.backtrace_field()?;
        distinct_backtrace_field(backtrace_field, self.from_field())
    }
}

impl Enum<'_> {
    pub(crate) fn has_source(&self) -> bool {
        self.variants
            .iter()
            .any(|variant| variant.source_field().is_some() || variant.attrs.transparent.is_some())
    }

    pub(crate) fn has_backtrace(&self) -> bool {
        self.variants
            .iter()
            .any(|variant| variant.backtrace_field().is_some())
    }

    pub(crate) fn has_display(&self) -> bool {
        self.attrs.display.is_some()
            || self.attrs.transparent.is_some()
            || self
                .variants
                .iter()
                .any(|variant| variant.attrs.display.is_some())
            || self
                .variants
                .iter()
                .all(|variant| variant.attrs.transparent.is_some())
    }
}

impl Variant<'_> {
    pub(crate) fn from_field(&self) -> Option<&Field> {
        from_field(&self.fields)
    }

    pub(crate) fn source_field(&self) -> Option<&Field> {
        source_field(&self.fields)
    }

    pub(crate) fn backtrace_field(&self) -> Option<&Field> {
        backtrace_field(&self.fields)
    }

    pub(crate) fn distinct_backtrace_field(&self) -> Option<&Field> {
        let backtrace_field = self.backtrace_field()?;
        distinct_backtrace_field(backtrace_field, self.from_field())
    }
}

impl Field<'_> {
    pub(crate) fn is_backtrace(&self) -> bool {
        type_is_backtrace(self.ty)
    }

    pub(crate) fn source_span(&self) -> Span {
        if let Some(source_attr) = &self.attrs.source {
            source_attr.path().get_ident().unwrap().span()
        } else if let Some(from_attr) = &self.attrs.from {
            from_attr.path().get_ident().unwrap().span()
        } else {
            self.member.member_span()
        }
    }
}

fn from_field<'a, 'b>(fields: &'a [Field<'b>]) -> Option<&'a Field<'b>> {
    fields.iter().find(|field| field.attrs.from.is_some())
}

fn source_field<'a, 'b>(fields: &'a [Field<'b>]) -> Option<&'a Field<'b>> {
    if let Some(field) = fields
        .iter()
        .find(|field| field.attrs.from.is_some() || field.attrs.source.is_some())
    {
        return Some(field);
    }
    if let Some(field) = fields.iter().find(|field| {
        matches!(&field.member,
            Member::Named(ident) if ident == "source"
        )
    }) {
        return Some(field);
    }
    None
}

fn backtrace_field<'a, 'b>(fields: &'a [Field<'b>]) -> Option<&'a Field<'b>> {
    if let Some(field) = fields.iter().find(|field| field.attrs.backtrace.is_some()) {
        return Some(field);
    }
    if let Some(field) = fields.iter().find(|field| field.is_backtrace()) {
        return Some(field);
    }
    None
}

// The #[backtrace] field, if it is not the same as the #[from] field.
fn distinct_backtrace_field<'a, 'b>(
    backtrace_field: &'a Field<'b>,
    from_field: Option<&Field>,
) -> Option<&'a Field<'b>> {
    if from_field.map_or(false, |from_field| {
        from_field.member == backtrace_field.member
    }) {
        None
    } else {
        Some(backtrace_field)
    }
}

fn type_is_backtrace(ty: &Type) -> bool {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return false,
    };

    let last = path.segments.last().unwrap();
    last.ident == "Backtrace" && last.arguments.is_empty()
}

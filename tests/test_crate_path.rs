//! Tests for `#[thiserror(crate = "...")]` — the attribute that lets a
//! consuming crate redirect the generated `::thiserror::__private::…` paths
//! to a re-exported copy of thiserror.

// Simulate the scenario where `thiserror` is only accessible through a
// re-exporting wrapper crate, not as a top-level dependency.
mod reexport {
    #[doc(hidden)]
    pub use thiserror;
    #[doc(hidden)]
    pub use thiserror::*;
}

// --- struct: basic re-export path ---

#[derive(reexport::Error, Debug)]
#[thiserror(crate = "reexport::thiserror")]
#[error("struct error: {msg}")]
struct StructError {
    msg: String,
}

#[test]
fn test_struct_display() {
    let e = StructError { msg: "boom".into() };
    assert_eq!(e.to_string(), "struct error: boom");
}

#[test]
fn test_struct_is_error() {
    let e = StructError { msg: "boom".into() };
    let _: &dyn std::error::Error = &e;
}

// --- enum: basic re-export path ---

#[derive(reexport::Error, Debug)]
#[thiserror(crate = "reexport::thiserror")]
enum EnumError {
    #[error("variant a")]
    A,
    #[error("variant b: {0}")]
    B(u32),
}

#[test]
fn test_enum_display_unit() {
    assert_eq!(EnumError::A.to_string(), "variant a");
}

#[test]
fn test_enum_display_tuple() {
    assert_eq!(EnumError::B(42).to_string(), "variant b: 42");
}

#[test]
fn test_enum_is_error() {
    let _: &dyn std::error::Error = &EnumError::A;
}

// --- explicit `::thiserror` path (same as default, exercises the code path) ---

#[derive(thiserror::Error, Debug)]
#[thiserror(crate = "::thiserror")]
#[error("explicit path error")]
struct ExplicitPathError;

#[test]
fn test_explicit_path_display() {
    assert_eq!(ExplicitPathError.to_string(), "explicit path error");
}

#[test]
fn test_explicit_path_is_error() {
    let _: &dyn std::error::Error = &ExplicitPathError;
}

// --- #[source] works through the re-export path ---

#[derive(reexport::Error, Debug)]
#[thiserror(crate = "reexport::thiserror")]
#[error("wrapper: {source}")]
struct WrapperError {
    #[source]
    source: StructError,
}

#[test]
fn test_source_chain() {
    use std::error::Error;

    let inner = StructError {
        msg: "inner".into(),
    };
    let outer = WrapperError { source: inner };
    assert_eq!(outer.to_string(), "wrapper: struct error: inner");
    assert!(outer.source().is_some());
}

// --- #[from] works through the re-export path ---

#[derive(reexport::Error, Debug)]
#[thiserror(crate = "reexport::thiserror")]
enum FromError {
    #[error("from struct: {0}")]
    FromStruct(#[from] StructError),
}

#[test]
fn test_from_impl() {
    let inner = StructError {
        msg: "via from".into(),
    };
    let outer = FromError::from(inner);
    assert_eq!(outer.to_string(), "from struct: struct error: via from");
}

// --- transparent forwarding works through the re-export path ---

#[derive(reexport::Error, Debug)]
#[thiserror(crate = "reexport::thiserror")]
#[error(transparent)]
struct TransparentError(StructError);

#[test]
fn test_transparent_display() {
    let e = TransparentError(StructError {
        msg: "inner".into(),
    });
    assert_eq!(e.to_string(), "struct error: inner");
}

#[test]
fn test_transparent_source() {
    use std::error::Error;
    // TransparentError delegates source() to StructError, which has no #[source]
    // field — so source() correctly returns None.
    let e = TransparentError(StructError {
        msg: "inner".into(),
    });
    assert!(e.source().is_none());
}

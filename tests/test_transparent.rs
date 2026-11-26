#![allow(clippy::items_after_statements)]

use anyhow::anyhow;
use std::error::Error as _;
use std::io;
use thiserror::Error;

#[test]
fn test_transparent_struct() {
    #[derive(Error, Debug)]
    #[error(transparent)]
    struct Error(ErrorKind);

    #[derive(Error, Debug)]
    enum ErrorKind {
        #[error("E0")]
        E0,
        #[error("E1")]
        E1(#[from] io::Error),
    }

    let error = Error(ErrorKind::E0);
    assert_eq!("E0", error.to_string());
    assert!(error.source().is_none());

    let io = io::Error::other("oh no!");
    let error = Error(ErrorKind::from(io));
    assert_eq!("E1", error.to_string());
    error.source().unwrap().downcast_ref::<io::Error>().unwrap();
}

#[test]
fn test_transparent_enum() {
    #[derive(Error, Debug)]
    enum Error {
        #[error("this failed")]
        This,
        #[error(transparent)]
        Other(anyhow::Error),
    }

    let error = Error::This;
    assert_eq!("this failed", error.to_string());

    let error = Error::Other(anyhow!("inner").context("outer"));
    assert_eq!("outer", error.to_string());
    assert_eq!("inner", error.source().unwrap().to_string());
}

#[test]
fn test_transparent_enum_with_default_message() {
    #[derive(Error, Debug)]
    #[error("this failed: {0}_{1}")]
    enum Error {
        This(i32, i32),
        #[error(transparent)]
        Other(anyhow::Error),
    }

    let error = Error::This(-1, -1);
    assert_eq!("this failed: -1_-1", error.to_string());

    let error = Error::Other(anyhow!("inner").context("outer"));
    assert_eq!("outer", error.to_string());
    assert_eq!("inner", error.source().unwrap().to_string());
}

#[test]
fn test_transparent_enum_generic() {
    #[derive(Error, Debug)]
    enum Error<E> {
        #[error("this failed")]
        This,
        #[error(transparent)]
        Other(E),
    }

    #[derive(Error, Debug)]
    #[error("inner error")]
    struct Inner;

    let error = Error::<Inner>::This;
    assert_eq!("this failed", error.to_string());

    let error = Error::Other(Inner);
    assert_eq!("inner error", error.to_string());
    assert!(error.source().is_none());

    #[derive(Error, Debug)]
    #[error("wrapped")]
    struct WithSource(#[source] io::Error);

    let io = io::Error::other("oh no!");
    let error = Error::Other(WithSource(io));
    assert_eq!("wrapped", error.to_string());
    assert_eq!("oh no!", error.source().unwrap().to_string());
}

#[test]
fn test_anyhow() {
    #[derive(Error, Debug)]
    #[error(transparent)]
    struct Any(#[from] anyhow::Error);

    let error = Any::from(anyhow!("inner").context("outer"));
    assert_eq!("outer", error.to_string());
    assert_eq!("inner", error.source().unwrap().to_string());
}

#[test]
fn test_non_static() {
    #[derive(Error, Debug)]
    #[error(transparent)]
    struct Error<'a> {
        inner: ErrorKind<'a>,
    }

    #[derive(Error, Debug)]
    enum ErrorKind<'a> {
        #[error("unexpected token: {:?}", token)]
        Unexpected { token: &'a str },
    }

    let error = Error {
        inner: ErrorKind::Unexpected { token: "error" },
    };
    assert_eq!("unexpected token: \"error\"", error.to_string());
    assert!(error.source().is_none());
}

#[test]
fn test_inherent_as_dyn_error_does_not_shadow_conversion() {
    #[derive(Debug)]
    struct Inner;

    impl std::fmt::Display for Inner {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("inner")
        }
    }

    #[derive(Debug)]
    struct InnerSource;

    impl std::fmt::Display for InnerSource {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("inner-source")
        }
    }

    impl std::error::Error for InnerSource {}

    impl std::error::Error for Inner {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            static SOURCE: InnerSource = InnerSource;
            Some(&SOURCE)
        }
    }

    #[derive(Debug)]
    struct Other;

    impl std::fmt::Display for Other {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("other")
        }
    }

    impl std::error::Error for Other {}

    impl Inner {
        // An inherent method with the same name as thiserror's hidden conversion
        // helper used to shadow it through method-call resolution.
        #[allow(dead_code)]
        fn as_dyn_error(&self) -> &(dyn std::error::Error + 'static) {
            static OTHER: Other = Other;
            &OTHER
        }
    }

    #[derive(Error, Debug)]
    #[error(transparent)]
    struct TransparentStruct(Inner);

    #[derive(Error, Debug)]
    enum TransparentEnum {
        #[error(transparent)]
        Inner(Inner),
    }

    #[derive(Error, Debug)]
    #[error("struct source")]
    struct SourceStruct {
        #[source]
        inner: Inner,
    }

    #[derive(Error, Debug)]
    enum SourceEnum {
        #[error("enum source")]
        Variant {
            #[source]
            inner: Inner,
        },
    }

    // For a transparent error, `source()` is forwarded to the field's own `source()`.
    assert_eq!(
        "inner-source",
        TransparentStruct(Inner).source().unwrap().to_string()
    );
    assert_eq!(
        "inner-source",
        TransparentEnum::Inner(Inner)
            .source()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "inner",
        SourceStruct { inner: Inner }.source().unwrap().to_string()
    );
    assert_eq!(
        "inner",
        SourceEnum::Variant { inner: Inner }
            .source()
            .unwrap()
            .to_string()
    );
}

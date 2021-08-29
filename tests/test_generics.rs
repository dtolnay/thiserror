#![deny(clippy::all, clippy::pedantic)]
#![allow(
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
    clippy::nonstandard_macro_braces,
)]
#![allow(dead_code)]

use std::fmt::{Debug, Display};

use thiserror::Error;

struct NoFormattingType;

struct DisplayType;

impl Display for DisplayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!(DisplayType))
    }
}

impl Debug for DisplayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!(DisplayType))
    }
}

struct DebugType;

impl Debug for DebugType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!(DebugType))
    }
}

/// Direct embedding of a generic in a field
///
/// Should produce the following instances:
///
/// ```rust
/// impl<Embedded> Display for DirectEmbedding<Embedded>
/// where
///     Embedded: Debug;
///
/// impl<Embedded> Error for DirectEmbedding<Embedded>
/// where
///     Self: Debug + Display;
/// ```
#[derive(Error, Debug)]
enum DirectEmbedding<Embedded> {
    #[error("{0:?}")]
    FatalError(Embedded),
}

/// #[from] handling but no Debug usage of the generic
///
/// Should produce the following instances:
///
/// ```rust
/// impl<Indirect> Display for FromGenericError<Indirect>;
///
/// impl<Indirect> Error for FromGenericError<Indirect>
/// where
///     DirectEmbedding<Indirect>: Error,
///     Indirect: 'static,
///     Self: Debug + Display;
/// ```
#[derive(Error, Debug)]
enum FromGenericError<Indirect> {
    #[error("Tadah")]
    SourceEmbedded(#[from] DirectEmbedding<Indirect>),
}

/// Direct embedding of a generic in a field
///
/// Should produce the following instances:
///
/// ```rust
/// impl<HasDisplay, HasDebug> Display for DirectEmbedding<HasDisplay, HasDebug>
/// where
///     HasDisplay: Display,
///     HasDebug: Debug;
///
/// impl<HasDisplay, HasDebug> Error for DirectEmbedding<HasDisplay, HasDebug>
/// where
///     Self: Debug + Display;
/// ```
#[derive(Error)]
enum HybridDisplayType<HasDisplay, HasDebug, HasNeither> {
    #[error("{0} : {1:?}")]
    HybridDisplayCase(HasDisplay, HasDebug),
    #[error("{0}")]
    DisplayCase(HasDisplay, HasNeither),
    #[error("{1:?}")]
    DebugCase(HasNeither, HasDebug),
}

impl<HasDisplay, HasDebug, HasNeither> Debug
    for HybridDisplayType<HasDisplay, HasDebug, HasNeither>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!(HybridDisplayType))
    }
}

fn display_hybrid_display_type(
    instance: HybridDisplayType<DisplayType, DebugType, NoFormattingType>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    Debug::fmt(&instance, f)
}

#[derive(Error, Debug)]
#[error("{0:?}")]
struct DirectEmbeddingStructTuple<Embedded>(Embedded);

#[derive(Error, Debug)]
#[error("{direct:?}")]
struct DirectEmbeddingStructNominal<Embedded> {
    direct: Embedded,
}

#[derive(Error, Debug)]
struct FromGenericErrorStructTuple<Indirect>(#[from] DirectEmbedding<Indirect>);

#[derive(Error, Debug)]
struct FromGenericErrorStructNominal<Indirect> {
    #[from]
    indirect: DirectEmbedding<Indirect>,
}

mod aserror;

pub use thiserror_impl::*;

// Not public API.
#[doc(hidden)]
pub mod private {
    pub use crate::aserror::AsDynError;
}

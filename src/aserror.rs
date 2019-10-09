use std::error::Error;

pub trait AsDynError {
    fn as_dyn_error(&self) -> &(dyn Error + 'static);
}

impl<T: Error + 'static> AsDynError for T {
    fn as_dyn_error(&self) -> &(dyn Error + 'static) {
        self
    }
}

impl AsDynError for dyn Error + 'static {
    fn as_dyn_error(&self) -> &(dyn Error + 'static) {
        self
    }
}

impl AsDynError for dyn Error + Send + Sync + 'static {
    fn as_dyn_error(&self) -> &(dyn Error + 'static) {
        self
    }
}

#![deny(unused_variables)]

pub enum ServerError {
    #[error("Connection timeout to {host}")]
    Timeout {
        host: String,
        // The `port` variable intentionally is not unsed in the macros string.
        // So, the compiler must throw unused_variables error
        port: u16,
    },
}

#[error("Failed to connect to {host}")]
pub struct ConnectionError {
    host: String,
    // Not used variable in this structure
    port: u16,
}

pub enum TupleError {
    #[error("First: {0}")]
    // The second element (u32) of the tuple is not used
    FirstOnly(String, u32),
}

fn main() {}

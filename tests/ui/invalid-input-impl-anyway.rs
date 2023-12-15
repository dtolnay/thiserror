use thiserror::Error;

#[derive(Error, Debug)]
#[error]
pub struct MyError;

fn main() {
    // FIXME: there should be no error on the following line. Thiserror should
    // emit an Error impl regardless of the bad attribute.
    _ = &MyError as &dyn std::error::Error;
}

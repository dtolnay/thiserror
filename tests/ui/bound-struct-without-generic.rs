use thiserror::Error;

#[derive(Error, Debug)]
#[error(bound = std::error::Error + 'static)]
struct BoundsWithoutGeneric {
    inner: u32,
}

impl std::fmt::Display for BoundsWithoutGeneric {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

fn main() {}

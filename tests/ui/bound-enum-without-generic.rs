use thiserror::Error;

#[derive(Error, Debug)]
#[error(bound = std::error::Error + 'static)]
enum BoundsWithoutGeneric {
    Variant(u32),
}

impl std::fmt::Display for BoundsWithoutGeneric {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

fn main() {}

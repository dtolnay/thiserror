use thiserror::Error;

#[derive(Debug, Error)]
#[error("{}", None::<i32>.is_some())]
struct Error;

#[test]
fn generic_unit_variant_format_arg_compiles() {
    let error = Error;
    assert_eq!(error.to_string(), "false");
}

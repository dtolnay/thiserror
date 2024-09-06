#[allow(unused_attributes)]
#[rustversion::attr(not(nightly), ignore = "requires nightly")]
#[cfg_attr(not(feature = "std"), ignore = "requires std")]
#[cfg_attr(miri, ignore = "incompatible with miri")]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

use library_fluent::hello_world;

/// Test using the library without using the provided
/// [`localizer()`](library_fluent::localizer()) method.
#[test]
fn test_no_localizer() {
    assert_eq!(&hello_world(), "Hello World!")
}

use twine_macros::define_component;

#[test]
fn test_define_component() {
    define_component! {
        name: example
    }

    // Verify that these structs now exist.
    let _ = example::Config;
    let _ = example::Input;
    let _ = example::Output;
}

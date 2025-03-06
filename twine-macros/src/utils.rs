use heck::ToUpperCamelCase;
use quote::format_ident;
use syn::Ident;

/// Extension trait for `Ident` to simplify common naming transformations.
pub(crate) trait IdentExt {
    /// Returns a new identifier in `UpperCamelCase`.
    fn upper_camel_case(&self) -> Ident;

    /// Returns a new identifier with the given prefix.
    fn with_prefix(&self, prefix: &str) -> Ident;

    /// Returns a new identifier with the given suffix.
    fn with_suffix(&self, suffix: &str) -> Ident;
}

impl IdentExt for Ident {
    fn upper_camel_case(&self) -> Ident {
        format_ident!("{}", self.to_string().to_upper_camel_case())
    }

    fn with_prefix(&self, prefix: &str) -> Ident {
        format_ident!("{}{}", prefix, self.to_string())
    }

    fn with_suffix(&self, suffix: &str) -> Ident {
        format_ident!("{}{}", self.to_string(), suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quote::format_ident;

    fn ident(name: &str) -> Ident {
        format_ident!("{}", name)
    }

    #[test]
    fn upper_camel_case_works() {
        assert_eq!(ident("single").upper_camel_case(), ident("Single"));
        assert_eq!(
            ident("example_name").upper_camel_case(),
            ident("ExampleName")
        );
        assert_eq!(
            ident("another_test_case").upper_camel_case(),
            ident("AnotherTestCase")
        );
        assert_eq!(
            ident("PascalCaseAlready").upper_camel_case(),
            ident("PascalCaseAlready")
        );
    }

    #[test]
    fn with_prefix_works() {
        assert_eq!(ident("example").with_prefix("test_"), ident("test_example"));
        assert_eq!(ident("FooBar").with_prefix("Test"), ident("TestFooBar"));
    }

    #[test]
    fn with_suffix_works() {
        assert_eq!(ident("example").with_suffix("_test"), ident("example_test"));
        assert_eq!(ident("FooBar").with_suffix("Test"), ident("FooBarTest"));
    }

    #[test]
    fn chaining_works() {
        assert_eq!(
            ident("example")
                .upper_camel_case()
                .with_prefix("Test")
                .with_suffix("Outputs"),
            ident("TestExampleOutputs")
        );
        assert_eq!(
            ident("the_model").upper_camel_case().with_suffix("Outputs"),
            ident("TheModelOutputs")
        );
        assert_eq!(
            ident("my_components")
                .with_prefix("all_")
                .with_suffix("_as_inputs")
                .upper_camel_case(),
            ident("AllMyComponentsAsInputs")
        );
    }
}

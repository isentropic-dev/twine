use std::{any::type_name, error::Error as StdError, fmt};

use crate::Component;

/// Represents an error in a [`Twine`] processing chain.
///
/// `TwineError` wraps errors from individual [`Component`]s while adding
/// contextual information about where the error occurred. It provides methods
/// for creating errors from components and attaching custom messages.
#[derive(Debug)]
pub struct TwineError {
    /// A descriptive message explaining the error.
    pub message: String,

    /// The underlying error that caused this failure.
    pub source: Box<dyn StdError + Send + Sync + 'static>,
}

impl TwineError {
    /// Creates a `TwineError` with a custom message and wrapped source error.
    ///
    /// Useful for adding context to errors in a [`Twine`] processing chain.
    ///
    /// # Parameters
    ///
    /// - `message`: A description of the error.
    /// - `error`: The underlying error being wrapped.
    pub fn new<E: StdError + Send + Sync + 'static>(message: String, error: E) -> Self {
        Self {
            message,
            source: Box::new(error),
        }
    }

    /// Extracts the type name of a failing component, includes it in an error
    /// message, and preserves the original error as the source.
    pub fn from_component<C: Component>(error: C::Error) -> Self {
        let full_type_name = type_name::<C>();
        let short_type = full_type_name.rsplit("::").next().unwrap_or(full_type_name);

        Self {
            message: format!("Component `{short_type}` failed: {error}"),
            source: Box::new(error),
        }
    }
}

impl fmt::Display for TwineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for TwineError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.source.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test error type to simulate a component error.
    #[derive(Debug)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Test error occurred")
        }
    }

    impl StdError for TestError {}

    /// A mock component that returns `TestError`.
    struct MockComponent;

    impl Component for MockComponent {
        type Input = ();
        type Output = ();
        type Error = TestError;

        fn call(&self, _input: Self::Input) -> Result<Self::Output, Self::Error> {
            Err(TestError)
        }
    }

    #[test]
    fn from_component_works() {
        let error = MockComponent.call(()).unwrap_err();

        let twine_error = TwineError::from_component::<MockComponent>(error);

        assert_eq!(
            twine_error.message,
            "Component `MockComponent` failed: Test error occurred",
        );

        assert_eq!(
            twine_error
                .source()
                .expect("Error will have a source")
                .to_string(),
            "Test error occurred"
        );
    }
}

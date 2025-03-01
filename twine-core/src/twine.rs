mod error;
mod function;
mod identity;

pub use error::TwineError;

use std::marker::PhantomData;

use crate::Component;

/// A builder for chaining components with consistent error handling.
///
/// `Twine` connects [`Component`]s sequentially, passing each output as the
/// next input. It wraps component errors in [`TwineError`] to ensure uniform
/// propagation and simplify composition.
///
/// # See Also
///
/// - [`Twine::<T>::new()`] — Starts a new chain.
/// - [`Twine::then()`] — Adds a component.
/// - [`Twine::then_fn()`] — Adds an inline function.
/// - [`Twine::build()`] — Finalizes the chain.
pub struct Twine<T, C = ()> {
    _marker: PhantomData<T>,
    component: C,
}

impl<T> Twine<T> {
    /// Creates a new `Twine` builder.
    ///
    /// The type `T`, which serves as the input type for the chain, must be
    /// specified using turbofish syntax (`Twine::<T>::new()`).
    #[must_use]
    pub fn new() -> Twine<T, impl Component<Input = T, Output = T, Error = TwineError>> {
        Twine {
            _marker: PhantomData,
            component: identity::Identity::new(),
        }
    }
}

impl<T, C: Component<Input = T, Error = TwineError>> Twine<T, C> {
    /// Adds a [`Component`] to the chain.
    ///
    /// The added component takes the current output as input, and its errors
    /// are wrapped in [`TwineError`] for consistent handling.
    ///
    /// # Example
    ///
    /// ```
    /// use std::convert::Infallible;
    /// use twine_core::{Component, Twine};
    ///
    /// struct Doubler;
    ///
    /// impl Component for Doubler {
    ///     type Input = i32;
    ///     type Output = i32;
    ///     type Error = Infallible;
    ///
    ///     fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
    ///         Ok(input * 2)
    ///     }
    /// }
    ///
    /// let chain = Twine::<i32>::new()
    ///     .then(Doubler)
    ///     .then(Doubler)
    ///     .build();
    ///
    /// assert_eq!(chain.call(2).unwrap(), 8);
    /// ```
    #[must_use]
    pub fn then<N>(
        self,
        next: N,
    ) -> Twine<T, impl Component<Input = T, Output = N::Output, Error = TwineError>>
    where
        N: Component<Input = C::Output>,
    {
        Twine {
            _marker: PhantomData,
            component: self
                .component
                .chain(next.map_error(|error| TwineError::from_component::<N>(error))),
        }
    }

    /// Adds an inline function to the chain.
    ///
    /// The added function takes the current output as its input and produces
    /// the next output in the chain.
    ///
    /// # Example
    ///
    /// ```
    /// use twine_core::{Component, Twine};
    ///
    /// let chain = Twine::<i32>::new()
    ///     .then_fn(|x| x + 10)
    ///     .then_fn(|x| x * 2)
    ///     .build();
    ///
    /// assert_eq!(chain.call(5).unwrap(), 30);
    /// ```
    #[must_use]
    pub fn then_fn<F, O>(
        self,
        function: F,
    ) -> Twine<T, impl Component<Input = T, Output = O, Error = TwineError>>
    where
        F: Fn(C::Output) -> O,
    {
        Twine {
            _marker: PhantomData,
            component: self.component.chain(function::Function::new(function)),
        }
    }

    /// Finalizes the chain and returns the composed [`Component`].
    ///
    /// The resulting component takes `T` as its input, returns the final output
    /// type, and uses [`TwineError`] as its error.
    #[must_use]
    pub fn build(self) -> impl Component<Input = T, Output = C::Output, Error = TwineError> {
        self.component
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::Infallible;

    /// A component that doubles the input.
    struct Doubler;
    impl Component for Doubler {
        type Input = i32;
        type Output = i32;
        type Error = Infallible;

        fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(input * 2)
        }
    }

    /// A component that adds some increment to the input.
    struct Adder {
        increment: i32,
    }
    impl Component for Adder {
        type Input = i32;
        type Output = i32;
        type Error = Infallible;

        fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(input + self.increment)
        }
    }

    /// A component that squares a floating point input.
    struct Squarer;
    impl Component for Squarer {
        type Input = f64;
        type Output = f64;
        type Error = Infallible;

        fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(input * input)
        }
    }

    /// A component that converts an integer to a string.
    struct IntToString;
    impl Component for IntToString {
        type Input = i32;
        type Output = String;
        type Error = Infallible;

        fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(format!("{input}"))
        }
    }

    #[test]
    fn call_a_single_component() {
        let chain = Twine::<i32>::new().then(Adder { increment: 10 }).build();
        assert_eq!(chain.call(0).unwrap(), 10);
        assert_eq!(chain.call(10).unwrap(), 20);
    }

    #[test]
    fn call_a_closure() {
        let chain = Twine::<i32>::new().then_fn(|x| x * 3).build();
        assert_eq!(chain.call(2).unwrap(), 6);
        assert_eq!(chain.call(5).unwrap(), 15);
    }

    #[test]
    fn chain_components() {
        let add_two = Adder { increment: 2 };
        let add_five = Adder { increment: 5 };

        let first_chain = Twine::<i32>::new().then(add_two).then(Doubler).build();
        assert_eq!(first_chain.call(0).unwrap(), 4);
        assert_eq!(first_chain.call(6).unwrap(), 16);

        let second_chain = Twine::<i32>::new().then(first_chain).then(add_five).build();
        assert_eq!(second_chain.call(1).unwrap(), 11);
    }

    #[test]
    fn chain_components_and_closures() {
        let add_ten = Adder { increment: 10 };
        let chain = Twine::<i32>::new()
            .then(add_ten)
            .then_fn(|x| x - 5)
            .then(Doubler)
            .then_fn(|x| x + 2)
            .build();

        assert_eq!(chain.call(0).unwrap(), 12);
        assert_eq!(chain.call(100).unwrap(), 212);
    }

    #[test]
    fn type_transformation() {
        let chain = Twine::<i32>::new().then(IntToString).build();
        assert_eq!(chain.call(42).unwrap(), "42".to_string());
        assert_eq!(chain.call(0).unwrap(), "0".to_string());
    }

    #[test]
    fn mixed_type_chaining() {
        let chain = Twine::<i32>::new()
            .then_fn(|x| x + 100)
            .then(IntToString)
            .then_fn(|s| format!("Value: {s}"))
            .build();

        assert_eq!(chain.call(0).unwrap(), "Value: 100");
        assert_eq!(chain.call(50).unwrap(), "Value: 150");
    }

    #[test]
    fn map_inside_then_to_use_a_context() {
        #[derive(Debug, PartialEq)]
        struct Context {
            input: f64,
            result: Option<String>,
        }

        let chain = Twine::<Context>::new()
            .then(Squarer.map(
                |&Context { input, .. }| input,
                |Context { input, .. }, output| Context {
                    input,
                    result: Some(format!("{input} squared is {output}")),
                },
            ))
            .then_fn(|Context { input, result }| Context {
                input,
                // We're very excited about this result.
                result: result.as_ref().map(|r| format!("{r}!")),
            })
            .build();

        let input = Context {
            input: 6.0,
            result: None,
        };

        let output = chain.call(input).unwrap();

        assert_eq!(output.result, Some("6 squared is 36!".into()));
    }
}

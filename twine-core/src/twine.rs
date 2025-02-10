mod closure;
mod then;

use std::marker::PhantomData;

use crate::Component;

/// A builder for composing multiple components into a single processing chain.
///
/// `Twine` enables sequential composition of [`Component`] implementations,
/// where each component's output serves as the next component's input.
///
/// # See Also
///
/// - [`Twine::new<T>()`] — Starts a new chain.
/// - [`Twine::then()`] — Adds any type that implements [`Component`].
/// - [`Twine::then_fn()`] — Adds an inline function.
/// - [`Twine::build()`] — Finalizes the chain and returns the composed component.
pub struct Twine<T, C = ()> {
    _marker: PhantomData<T>,
    component: C,
}

impl<T> Twine<T> {
    /// Starts a new `Twine` builder with `T` as the initial input type.
    ///
    /// This function creates an empty chain where `T` serves as the starting input.
    /// Components added via [`Twine::then()`] or [`Twine::then_fn()`] can
    /// progressively transform the data as it moves through the chain.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the first component to the chain.
    ///
    /// See [`Twine::then()`] for details.
    #[must_use]
    pub fn then<C>(self, component: C) -> Twine<T, C>
    where
        C: Component<Input = T>,
    {
        Twine {
            _marker: PhantomData,
            component,
        }
    }

    /// Adds an inline function as the first step in the chain.
    ///
    /// See [`Twine::then_fn()`] for details.
    #[must_use]
    pub fn then_fn<F, O>(self, function: F) -> Twine<T, impl Component<Input = T, Output = O>>
    where
        F: Fn(&T) -> O,
    {
        Twine {
            _marker: PhantomData,
            component: closure::Closure::new(function),
        }
    }
}

impl<T, C: Component<Input = T>> Twine<T, C> {
    /// Adds a component to the chain.
    ///
    /// This method appends a [`Component`] to the sequence, using the current
    /// output as its input.
    ///
    /// # Parameters
    ///
    /// - `next`: The [`Component`] that processes the output of the current chain.
    ///
    /// # Example
    ///
    /// ```
    /// use twine_core::{Component, Twine};
    ///
    /// struct Doubler;
    ///
    /// impl Component for Doubler {
    ///     type Input = i32;
    ///     type Output = i32;
    ///
    ///     fn call(&self, input: &Self::Input) -> Self::Output {
    ///         input * 2
    ///     }
    /// }
    ///
    /// let chain = Twine::<i32>::new()
    ///     .then(Doubler)
    ///     .build();
    ///
    /// assert_eq!(chain.call(&2), 4);
    /// ```
    #[must_use]
    pub fn then<N>(self, next: N) -> Twine<T, impl Component<Input = C::Input, Output = N::Output>>
    where
        N: Component<Input = C::Output>,
    {
        Twine {
            _marker: PhantomData,
            component: then::Then::new(self.component, next),
        }
    }

    /// Adds an inline function to the chain.
    ///
    /// This method applies a function to the output of the current component
    /// before passing it to the next component or function.
    ///
    /// # Parameters
    ///
    /// - `next`: A function that processes the output of the current chain.
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
    /// assert_eq!(chain.call(&5), 30);
    /// ```
    #[must_use]
    pub fn then_fn<F, O>(
        self,
        function: F,
    ) -> Twine<T, impl Component<Input = C::Input, Output = O>>
    where
        F: Fn(&C::Output) -> O,
    {
        Twine {
            _marker: PhantomData,
            component: then::Then::new(self.component, closure::Closure::new(function)),
        }
    }

    /// Finalizes the `Twine` chain and returns the composed component.
    ///
    /// This method completes the chain-building process, producing a
    /// [`Component`] that can be executed with `.call(input)`.
    #[must_use]
    pub fn build(self) -> impl Component<Input = C::Input, Output = C::Output> {
        self.component
    }
}

impl<T> Default for Twine<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
            component: (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A component that doubles the input.
    struct Doubler;
    impl Component for Doubler {
        type Input = i32;
        type Output = i32;

        fn call(&self, input: &Self::Input) -> Self::Output {
            input * 2
        }
    }

    /// A component that adds some increment to the input.
    struct Adder {
        increment: i32,
    }
    impl Component for Adder {
        type Input = i32;
        type Output = i32;

        fn call(&self, input: &Self::Input) -> Self::Output {
            input + self.increment
        }
    }

    /// A component that squares a floating point input.
    struct Squarer;
    impl Component for Squarer {
        type Input = f64;
        type Output = f64;

        fn call(&self, input: &Self::Input) -> Self::Output {
            input * input
        }
    }

    /// A component that converts an integer to a string.
    struct IntToString;
    impl Component for IntToString {
        type Input = i32;
        type Output = String;

        fn call(&self, input: &Self::Input) -> Self::Output {
            format!("{input}")
        }
    }

    #[test]
    fn call_a_single_component() {
        let chain = Twine::<i32>::new().then(Adder { increment: 10 }).build();
        assert_eq!(chain.call(&0), 10);
        assert_eq!(chain.call(&10), 20);
    }

    #[test]
    fn call_a_closure() {
        let chain = Twine::<i32>::new().then_fn(|x| x * 3).build();
        assert_eq!(chain.call(&2), 6);
        assert_eq!(chain.call(&5), 15);
    }

    #[test]
    fn chain_components() {
        let add_two = Adder { increment: 2 };
        let add_five = Adder { increment: 5 };

        let first_chain = Twine::<i32>::new().then(add_two).then(Doubler).build();
        assert_eq!(first_chain.call(&0), 4);
        assert_eq!(first_chain.call(&6), 16);

        let second_chain = Twine::<i32>::new().then(first_chain).then(add_five).build();
        assert_eq!(second_chain.call(&1), 11);
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

        assert_eq!(chain.call(&0), 12);
        assert_eq!(chain.call(&100), 212);
    }

    #[test]
    fn type_transformation() {
        let chain = Twine::<i32>::new().then(IntToString).build();
        assert_eq!(chain.call(&42), "42".to_string());
        assert_eq!(chain.call(&0), "0".to_string());
    }

    #[test]
    fn mixed_type_chaining() {
        let chain = Twine::<i32>::new()
            .then_fn(|x| x + 100)
            .then(IntToString)
            .then_fn(|s| format!("Value: {s}"))
            .build();

        assert_eq!(chain.call(&0), "Value: 100");
        assert_eq!(chain.call(&50), "Value: 150");
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
                |(Context { input, .. }, output)| Context {
                    input: *input,
                    result: Some(format!("{input} squared is {output}")),
                },
            ))
            .then_fn(|Context { input, result }| Context {
                input: *input,
                // We're very excited about this result.
                result: result.as_ref().map(|r| format!("{r}!")),
            })
            .build();

        let input = Context {
            input: 6.0,
            result: None,
        };

        let output = chain.call(&input);

        assert_eq!(output.result, Some("6 squared is 36!".into()));
    }
}

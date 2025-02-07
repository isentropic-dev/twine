mod closure;
mod identity;
mod then;

use crate::Component;

/// A builder for composing multiple components into a single, executable chain.
///
/// `Twine` enables structured composition of [`Component`] implementations,
/// where each component’s output becomes the next component’s input.
///
/// Use [`Twine::new<T>()`] to start a chain with a specified input type.
/// Components that implement [`Component`] can be added with [`Twine::then()`],
/// while closures can be added with [`Twine::then_fn()`].
///
/// Once the chain is complete, call [`Twine::build()`] to return the final composed component.
///
/// # See Also
///
/// - [`Twine::new<T>()`] — Starts a new chain.
/// - [`Twine::then()`] — Adds a component that implements [`Component`].
/// - [`Twine::then_fn()`] — Adds an inline closure.
/// - [`Twine::build()`] — Finalizes the chain and returns the composed component.
pub struct Twine<C> {
    component: C,
}

impl Twine<()> {
    /// Starts a new `Twine` chain.
    ///
    /// This method begins a new component chain and defines the input type of
    /// the final composed component.
    ///
    /// Use [`Twine::then()`] to add components or [`Twine::then_fn()`] to add closures.
    /// Once the chain is complete, call [`Twine::build()`] to finalize it.
    ///
    /// # Example
    ///
    /// ```
    /// use twine_core::{Component, Twine};
    ///
    /// let chain = Twine::new::<i32>()
    ///     .then_fn(|x| x + 7)
    ///     .then_fn(|x| x * 2)
    ///     .build();
    ///
    /// assert_eq!(chain.call(6), 26);
    /// ```
    #[must_use]
    pub fn new<T>() -> Twine<impl Component<Input = T, Output = T>> {
        Twine {
            component: identity::Identity::<T>::new(),
        }
    }
}

impl<C: Component> Twine<C> {
    /// Adds a `Component` to the chain.
    ///
    /// This method extends the `Twine` sequence by adding another `Component`,
    /// where the current component’s output becomes the input for the new one.
    ///
    /// Use this method when `next` is a type that implements [`Component`]. To
    /// add an inline closure instead, use [`Twine::then_fn()`].
    ///
    /// # Parameters
    ///
    /// - `next`: A `Component` that processes the output of the current chain.
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
    ///     fn call(&self, input: Self::Input) -> Self::Output {
    ///         input * 2
    ///     }
    /// }
    ///
    /// let chain = Twine::new::<i32>()
    ///     .then(Doubler)
    ///     .build();
    ///
    /// assert_eq!(chain.call(2), 4);
    /// ```
    #[must_use]
    pub fn then<N>(self, next: N) -> Twine<impl Component<Input = C::Input, Output = N::Output>>
    where
        N: Component<Input = C::Output>,
    {
        Twine {
            component: then::Then::new(self.component, next),
        }
    }

    /// Adds an inline function to the chain.
    ///
    /// This method extends the `Twine` sequence by applying a function to the
    /// output of the current component before passing it to the next.
    ///
    /// Use this method when `next` is a function or closure. To add a full
    /// `Component`, use [`Twine::then()`].
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
    /// let chain = Twine::new::<i32>()
    ///     .then_fn(|x| x + 10)
    ///     .then_fn(|x| x * 2)
    ///     .build();
    ///
    /// assert_eq!(chain.call(5), 30);
    /// ```
    #[must_use]
    pub fn then_fn<N, O>(self, next: N) -> Twine<impl Component<Input = C::Input, Output = O>>
    where
        N: Fn(C::Output) -> O,
    {
        Twine {
            component: then::Then::new(self.component, closure::Closure::new(next)),
        }
    }

    /// Finalizes the `Twine` chain and returns the composed component.
    ///
    /// This method completes the chain-building process and produces a
    /// `Component` that can be executed with `.call(input)`.
    ///
    /// The resulting `Component` retains all transformations and can be used
    /// independently of `Twine`.
    #[must_use]
    pub fn build(self) -> impl Component<Input = C::Input, Output = C::Output> {
        self.component
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

        fn call(&self, input: Self::Input) -> Self::Output {
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

        fn call(&self, input: Self::Input) -> Self::Output {
            input + self.increment
        }
    }

    /// A component that squares the input.
    struct Squarer;
    impl Component for Squarer {
        type Input = f64;
        type Output = f64;

        fn call(&self, input: Self::Input) -> Self::Output {
            input * input
        }
    }

    /// A component that converts an integer to a string.
    struct IntToString;
    impl Component for IntToString {
        type Input = i32;
        type Output = String;

        fn call(&self, input: Self::Input) -> Self::Output {
            format!("{input}")
        }
    }

    #[test]
    fn identity_component() {
        let chain = Twine::new::<i32>().build();
        assert_eq!(chain.call(5), 5);
        assert_eq!(chain.call(-10), -10);
    }

    #[test]
    fn call_a_single_component() {
        let chain = Twine::new::<i32>().then(Adder { increment: 10 }).build();
        assert_eq!(chain.call(0), 10);
        assert_eq!(chain.call(10), 20);
    }

    #[test]
    fn call_a_closure() {
        let chain = Twine::new::<i32>().then_fn(|x| x * 3).build();
        assert_eq!(chain.call(2), 6);
        assert_eq!(chain.call(5), 15);
    }

    #[test]
    fn chain_components() {
        let add_two = Adder { increment: 2 };
        let add_five = Adder { increment: 5 };

        let first_chain = Twine::new::<i32>().then(add_two).then(Doubler).build();
        assert_eq!(first_chain.call(0), 4);
        assert_eq!(first_chain.call(6), 16);

        let second_chain = Twine::new::<i32>().then(first_chain).then(add_five).build();
        assert_eq!(second_chain.call(1), 11);
    }

    #[test]
    fn chain_components_and_closures() {
        let add_ten = Adder { increment: 10 };
        let chain = Twine::new::<i32>()
            .then(add_ten)
            .then_fn(|x| x - 5)
            .then(Doubler)
            .then_fn(|x| x + 2)
            .build();

        assert_eq!(chain.call(0), 12);
        assert_eq!(chain.call(100), 212);
    }

    #[test]
    fn type_transformation() {
        let chain = Twine::new::<i32>().then(IntToString).build();
        assert_eq!(chain.call(42), "42".to_string());
        assert_eq!(chain.call(0), "0".to_string());
    }

    #[test]
    fn mixed_type_chaining() {
        let chain = Twine::new::<i32>()
            .then_fn(|x| x + 100)
            .then(IntToString)
            .then_fn(|s| format!("Value: {s}"))
            .build();

        assert_eq!(chain.call(0), "Value: 100");
        assert_eq!(chain.call(50), "Value: 150");
    }

    #[test]
    fn map_inside_then_to_use_a_context() {
        #[derive(Debug, PartialEq)]
        struct Context {
            input: f64,
            result: Option<String>,
        }

        let chain = Twine::new::<Context>()
            .then(Squarer.map(
                |&Context { input, .. }| input,
                |(Context { input, .. }, output)| Context {
                    input,
                    result: Some(format!("{input} squared is {output}")),
                },
            ))
            .then_fn(|Context { input, result }| Context {
                input,
                // We're very excited about this result.
                result: result.map(|r| format!("{r}!")),
            })
            .build();

        let input = Context {
            input: 6.0,
            result: None,
        };

        let output = chain.call(input);

        assert_eq!(output.result, Some("6 squared is 36!".into()));
    }
}

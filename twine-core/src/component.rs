mod inspect;
mod mapped;
mod mapped_error;
mod then;

/// The core trait for defining components in Twine.
///
/// A `Component` transforms an input into an output and serves as the
/// foundation for composition in Twine. It enables structured, reusable
/// processing units that can be combined, adapted, and extended.
///
/// Implementations must be deterministic, meaning that the component always
/// produces the same output given the same input.
///
/// Components can be sequentially composed using [`Component::then()`], which
/// chains execution while ensuring type safety. To chain components:
/// - The first component’s output type must match the second’s input type.
/// - Both components must share the same error type.
///
/// Components can also be adapted with:
/// - [`Component::map()`] to transform inputs and outputs.
/// - [`Component::map_error()`] to transform error types.
/// - [`Component::inspect()`] to observe execution without altering behavior.
pub trait Component {
    type Input;
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Calls the component with the given input, producing an output or an error.
    ///
    /// # Errors
    ///
    /// Returns an error of type [`Component::Error`] if the call fails,
    /// allowing components to manage their own errors.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;

    /// Transforms the component’s input and output.
    ///
    /// This method adapts a component to integrate into a broader context:
    /// - `input_map` extracts the expected input type.
    /// - `output_map` integrates the component’s output back into the original context.
    ///
    /// If the component produces an error, it is returned unchanged.
    ///
    /// # Parameters
    ///
    /// - `input_map`: Extracts the component's expected input from a broader context.
    /// - `output_map`: Integrates the component's output back into the original context.
    ///
    /// # Returns
    ///
    /// A new component with modified input and output behavior, preserving the error type.
    ///
    /// # Example
    ///
    /// ```
    /// use std::convert::Infallible;
    /// use twine_core::Component;
    ///
    /// struct Adder {
    ///     increment: i32,
    /// }
    ///
    /// impl Component for Adder {
    ///     type Input = i32;
    ///     type Output = i32;
    ///     type Error = Infallible;
    ///
    ///     fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
    ///         Ok(input + self.increment)
    ///     }
    /// }
    ///
    /// struct Input {
    ///     value: i32,
    ///     other_data: f64,
    /// }
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Output {
    ///     started_with: i32,
    ///     ended_with: i32,
    ///     is_even: bool,
    ///     other_data: f64,
    /// }
    ///
    /// let add_five = Adder { increment: 5 };
    ///
    /// let mapped_add_five = add_five.map(
    ///     // Destructuring is often useful here.
    ///     |&Input { value, .. }| value,
    ///     |Input {value, other_data }, output| Output {
    ///         started_with: value,
    ///         ended_with: output,
    ///         is_even: output % 2 == 0,
    ///         other_data,
    ///     },
    /// );
    ///
    /// let input = Input { value: 3, other_data: 100.0 };
    ///
    /// assert_eq!(
    ///     mapped_add_five.call(input),
    ///     Ok(Output {
    ///         started_with: 3,
    ///         ended_with: 8,
    ///         is_even: true,
    ///         other_data: 100.0,
    ///     })
    /// );
    /// ```
    fn map<InputMap, OutputMap, In, Out>(
        self,
        input_map: InputMap,
        output_map: OutputMap,
    ) -> impl Component<Input = In, Output = Out, Error = Self::Error>
    where
        Self: Sized,
        InputMap: Fn(&In) -> Self::Input,
        OutputMap: Fn(In, Self::Output) -> Out,
    {
        mapped::Mapped::new(self, input_map, output_map)
    }

    /// Transforms the component’s error into a different type.
    ///
    /// Converts low-level errors into structured, higher-level errors for
    /// better integration within a broader context.
    ///
    /// # Returns
    ///
    /// A new component with the same input and output types but a transformed error type.
    fn map_error<ErrorMap, NewError>(
        self,
        error_map: ErrorMap,
    ) -> impl Component<Input = Self::Input, Output = Self::Output, Error = NewError>
    where
        Self: Sized,
        ErrorMap: Fn(Self::Error) -> NewError,
        NewError: std::error::Error + Send + Sync + 'static,
    {
        mapped_error::MappedError::new(self, error_map)
    }

    /// Observes input and output without modifying the component’s behavior.
    ///
    /// # Parameters
    ///
    /// - `input_handler`: Called before execution to inspect the input.
    /// - `output_handler`: Called after execution to inspect the output.
    ///
    /// # Returns
    ///
    /// A new component that calls the handlers but otherwise behaves the same.
    ///
    /// # Example
    ///
    /// ```
    /// use std::convert::Infallible;
    /// use twine_core::Component;
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
    /// let debug_component = Doubler.inspect(
    ///     |input| println!("Received: {:?}", input),
    ///     |output| println!("Produced: {:?}", output),
    /// );
    ///
    /// debug_component.call(5);
    /// // Prints:
    /// // Received: 5
    /// // Produced: 10
    /// ```
    fn inspect<InputHandler, OutputHandler>(
        self,
        input_handler: InputHandler,
        output_handler: OutputHandler,
    ) -> impl Component<Input = Self::Input, Output = Self::Output, Error = Self::Error>
    where
        Self: Sized,
        InputHandler: Fn(&Self::Input),
        OutputHandler: Fn(&Self::Output),
    {
        inspect::Inspect {
            component: self,
            input_handler,
            output_handler,
        }
    }

    /// Chains this component with another.
    ///
    /// The second component must accept this component’s output as input and
    /// share the same error type, ensuring type-safe composition.
    ///
    /// # Returns
    ///
    /// A new component that executes `self` and passes its output to `next`.
    ///
    /// # Example
    /// ```
    /// use std::convert::Infallible;
    /// use twine_core::Component;
    ///
    /// struct Double;
    /// impl Component for Double {
    ///     type Input = i32;
    ///     type Output = i32;
    ///     type Error = Infallible;
    ///
    ///     fn call(&self, input: i32) -> Result<i32, Self::Error> {
    ///         Ok(input * 2)
    ///     }
    /// }
    ///
    /// struct Increment;
    /// impl Component for Increment {
    ///     type Input = i32;
    ///     type Output = i32;
    ///     type Error = Infallible;
    ///
    ///     fn call(&self, input: i32) -> Result<i32, Self::Error> {
    ///         Ok(input + 1)
    ///     }
    /// }
    ///
    /// let chain = Double.then(Increment);
    /// assert_eq!(chain.call(3).unwrap(), 7);
    /// ```
    fn then<B>(
        self,
        next: B,
    ) -> impl Component<Input = Self::Input, Output = B::Output, Error = Self::Error>
    where
        Self: Sized,
        B: Component<Input = Self::Output, Error = Self::Error>,
    {
        then::Then {
            first: self,
            second: next,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{convert::Infallible, error::Error as StdError, fmt};

    use super::*;

    struct Doubler;

    impl Component for Doubler {
        type Input = i32;
        type Output = i32;
        type Error = Infallible;

        fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(input * 2)
        }
    }

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

    struct Failer;

    impl Component for Failer {
        type Input = ();
        type Output = ();
        type Error = FailerError;

        fn call(&self, _input: Self::Input) -> Result<Self::Output, Self::Error> {
            Err(FailerError)
        }
    }

    #[derive(Debug)]
    struct FailerError;

    impl fmt::Display for FailerError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "The failer failed.")
        }
    }

    impl StdError for FailerError {}

    #[test]
    fn basic_components() {
        assert_eq!(Doubler.call(2), Ok(4));
        assert_eq!(Doubler.call(5), Ok(10));

        let add_one = Adder { increment: 1 };
        assert_eq!(add_one.call(10), Ok(11));

        let add_five = Adder { increment: 5 };
        assert_eq!(add_five.call(3), Ok(8));
    }

    #[test]
    fn mapped_component_to_string() {
        let mapped = Doubler.map(
            |&input| input + 1,
            |input, output| format!("Adding 1 to {input} and doubling it is {output}"),
        );

        assert_eq!(
            mapped.call(2).unwrap(),
            "Adding 1 to 2 and doubling it is 6"
        );
    }

    #[test]
    fn mapped_component_with_single_context() {
        #[derive(Debug, PartialEq, Eq)]
        struct Context {
            input: i32,
            output: i32,
        }

        let mapped_doubler = Doubler.map(
            |context: &Context| context.input,
            |context, output| Context {
                input: context.input,
                output,
            },
        );

        let context_in = Context {
            input: 10,
            output: 0,
        };

        let context_out = mapped_doubler.call(context_in).unwrap();

        assert_eq!(
            context_out,
            Context {
                input: 10,
                output: 20
            }
        );
    }

    #[test]
    fn mapped_component_that_changes_context_type() {
        #[derive(Debug, PartialEq, Eq)]
        struct MyInput {
            label: String,
            value: i32,
        }

        #[derive(Debug, PartialEq, Eq)]
        struct MyOutput {
            label: String,
            started_with: i32,
            ended_with: i32,
            is_even: bool,
        }

        let add_three = Adder { increment: 3 };

        let mapped_add_three = add_three.map(
            // Destructuring is often useful here.
            |&MyInput {
                 value: value_to_use,
                 ..
             }| value_to_use,
            |input, output| MyOutput {
                label: input.label.clone(),
                started_with: input.value,
                ended_with: output,
                is_even: output % 2 == 0,
            },
        );

        let input = MyInput {
            label: "A label".into(),
            value: 3,
        };

        let output = mapped_add_three.call(input).unwrap();

        assert_eq!(
            output,
            MyOutput {
                label: "A label".into(),
                started_with: 3,
                ended_with: 6,
                is_even: true,
            }
        );
    }

    #[test]
    fn map_error_transforms_component_error() {
        use std::fmt;

        #[derive(Debug)]
        struct MappedError(String);

        impl fmt::Display for MappedError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl StdError for MappedError {}

        let will_fail = Failer
            .map_error(|err| MappedError(format!("The wrapped component failed with: {err}")));

        let result = will_fail.call(());

        assert_eq!(
            result.unwrap_err().to_string(),
            "The wrapped component failed with: The failer failed."
        );
    }

    #[test]
    fn inspect_component() {
        use std::sync::{Arc, Mutex};

        // Typically `.inspect()` will be used with `println!()`, which can be
        // called inline in the handlers. This extra setup is only needed to
        // capture values for assertions in this test.
        let input_log = Arc::new(Mutex::new(Vec::new()));
        let output_log = Arc::new(Mutex::new(Vec::new()));

        let inspected = Doubler.inspect(
            {
                let input_log = Arc::clone(&input_log);
                move |input| input_log.lock().unwrap().push(*input)
            },
            {
                let output_log = Arc::clone(&output_log);
                move |output| output_log.lock().unwrap().push(*output)
            },
        );

        let result1 = inspected.call(3).unwrap();
        let result2 = inspected.call(5).unwrap();

        assert_eq!(result1, 6);
        assert_eq!(result2, 10);

        assert_eq!(*input_log.lock().unwrap(), vec![3, 5]);
        assert_eq!(*output_log.lock().unwrap(), vec![6, 10]);
    }

    #[test]
    fn chain_components_with_then() {
        let add_one = Adder { increment: 1 };
        let add_ten = Adder { increment: 10 };
        let chain = add_one.then(Doubler).then(add_ten);

        assert_eq!(chain.call(2), Ok(16));
        assert_eq!(chain.call(20), Ok(52));
    }
}

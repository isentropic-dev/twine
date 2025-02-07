mod inspect;
mod mapped;

/// The core trait for defining components in Twine.
///
/// A `Component` represents a transformation from an input to an output
/// and serves as the foundation for composition in Twine. Components can be
/// composed using [`Twine`] to build sequential processing chains.
///
/// Implementations of `Component` must be deterministic, meaning that the same
/// input must always produce the same output.
///
/// Components can be adapted using [`Component::map()`] to modify input/output
/// behavior or observed using [`Component::inspect()`] for debugging.
pub trait Component {
    type Input;
    type Output;

    /// Calls the component with the given input.
    ///
    /// Executes the component, consuming an input and producing an output.
    fn call(&self, input: Self::Input) -> Self::Output;

    /// Adapts this component by transforming its input and output.
    ///
    /// This method enables context-aware composition, allowing a component
    /// to be embedded within a broader structure. The `input_map` function
    /// derives the component's input from a larger context, while `output_map`
    /// integrates its output back into that context.
    ///
    /// # Parameters
    ///
    /// - `input_map`: Extracts the expected input from a larger structure.
    /// - `output_map`: Combines the original input with the component's output.
    ///
    /// # Returns
    ///
    /// - A new component that integrates `Self` into a different input/output structure.
    ///
    /// # Example
    ///
    /// ```
    /// use twine_core::Component;
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
    /// struct Context {
    ///     value: i32,
    ///     doubled: i32,
    /// }
    ///
    /// let component = Doubler.map(
    ///     |context: &Context| context.value,
    ///     |(context, output)| Context {
    ///         value: context.value,
    ///         doubled: output,
    ///     }
    /// );
    ///
    /// let result = component.call(Context { value: 4, doubled: 0 });
    /// assert_eq!(result.doubled, 8);
    /// ```
    fn map<InputMap, OutputMap, In, Out>(
        self,
        input_map: InputMap,
        output_map: OutputMap,
    ) -> impl Component<Input = In, Output = Out>
    where
        Self: Sized,
        InputMap: Fn(&In) -> Self::Input,
        OutputMap: Fn((In, Self::Output)) -> Out,
    {
        mapped::Mapped::new(self, input_map, output_map)
    }

    /// Observes input and output without modifying computation.
    ///
    /// This method wraps the component and invokes the provided handlers before
    /// and after calling the component. It does not alter the component’s logic
    /// but allows logging, debugging, or tracing behavior.
    ///
    /// The `input_handler` is called before the component processes the input,
    /// while  the `output_handler` is called after the component has produced
    /// an output. Both handlers receive borrowed references to the values,
    /// ensuring no ownership changes.
    ///
    /// # Parameters
    ///
    /// - `input_handler`: A function that inspects the input before processing.
    /// - `output_handler`: A function that inspects the output after processing.
    ///
    /// # Returns
    ///
    /// - A new component that behaves identically but calls the handlers on each execution.
    ///
    /// # Example
    ///
    /// ```
    /// use twine_core::Component;
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
    ) -> impl Component<Input = Self::Input, Output = Self::Output>
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
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Doubler;

    impl Component for Doubler {
        type Input = i32;
        type Output = i32;

        fn call(&self, input: Self::Input) -> Self::Output {
            input * 2
        }
    }

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

    #[test]
    fn basic_components() {
        assert_eq!(Doubler.call(2), 4);
        assert_eq!(Doubler.call(5), 10);

        let add_one = Adder { increment: 1 };
        assert_eq!(add_one.call(10), 11);

        let add_five = Adder { increment: 5 };
        assert_eq!(add_five.call(3), 8);
    }

    #[test]
    fn mapped_component_to_string() {
        let mapped = Doubler.map(
            |&input| input + 1,
            |(input, output)| format!("Adding 1 to {input} and doubling it is {output}"),
        );

        assert_eq!(mapped.call(2), "Adding 1 to 2 and doubling it is 6");
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
            |(context, output)| Context {
                input: context.input,
                output,
            },
        );

        let context_in = Context {
            input: 10,
            output: 0,
        };

        let context_out = mapped_doubler.call(context_in);

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
            |(input, output)| MyOutput {
                label: input.label,
                started_with: input.value,
                ended_with: output,
                is_even: output % 2 == 0,
            },
        );

        let input = MyInput {
            label: "A label".into(),
            value: 3,
        };

        let output = mapped_add_three.call(input);

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

        let result1 = inspected.call(3);
        let result2 = inspected.call(5);

        assert_eq!(result1, 6);
        assert_eq!(result2, 10);

        assert_eq!(*input_log.lock().unwrap(), vec![3, 5]);
        assert_eq!(*output_log.lock().unwrap(), vec![6, 10]);
    }
}

use super::Component;

/// A wrapper that observes input and output without modifying behavior.
///
/// This struct is used internally by `.inspect()`.
pub(crate) struct Inspect<C, InputHandler, OutputHandler> {
    pub(crate) component: C,
    pub(crate) input_handler: InputHandler,
    pub(crate) output_handler: OutputHandler,
}

impl<C, InputHandler, OutputHandler> Component for Inspect<C, InputHandler, OutputHandler>
where
    C: Component,
    InputHandler: Fn(&C::Input),
    OutputHandler: Fn(&C::Output),
{
    type Input = C::Input;
    type Output = C::Output;

    fn call(&self, input: Self::Input) -> Self::Output {
        (self.input_handler)(&input);
        let output = self.component.call(input);
        (self.output_handler)(&output);
        output
    }
}

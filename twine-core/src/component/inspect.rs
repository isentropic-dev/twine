use super::Component;

/// A wrapper that observes input and output without modifying behavior.
///
/// Internally used by `.inspect()` to observe component execution.
pub(crate) struct Inspect<C, InputHandler, OutputHandler>
where
    C: Component,
    InputHandler: Fn(&C::Input),
    OutputHandler: Fn(&C::Output),
{
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
    type Error = C::Error;

    /// Calls the wrapped component while observing its input and output.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        (self.input_handler)(&input);
        let output = self.component.call(input)?;
        (self.output_handler)(&output);
        Ok(output)
    }
}

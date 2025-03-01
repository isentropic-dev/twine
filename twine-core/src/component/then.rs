use crate::Component;

/// A wrapper that calls two components sequentially.
///
/// Internally used by `.then()` to chain two compatible components.
///
/// For components to be compatible, the first component’s output type must
/// match the second’s input, enabling type-safe composition. Both components
/// must share the same error type, ensuring errors propagate unchanged.
pub(crate) struct Then<A, B>
where
    A: Component,
    B: Component<Input = A::Output, Error = A::Error>,
{
    pub(crate) first: A,
    pub(crate) second: B,
}

impl<A, B> Component for Then<A, B>
where
    A: Component,
    B: Component<Input = A::Output, Error = A::Error>,
{
    type Input = A::Input;
    type Output = B::Output;
    type Error = A::Error;

    /// Calls the first component and passes its output to the second.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let output = self.first.call(input)?;
        self.second.call(output)
    }
}

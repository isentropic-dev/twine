use crate::Component;

/// A component that calls two components sequentially.
///
/// This struct is used internally by `.then()` to chain two compatible
/// components together.
///
/// `Then` ensures that the first component’s output type matches the second
/// component’s input type, enabling type-safe composition. When `call()` is
/// invoked, the input is processed by the first component (`A`), and its output
/// is passed as input to the second component (`B`).
///
/// Both components must share the same error type (`A::Error`), ensuring that
/// errors propagate naturally without modification.
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

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let output = self.first.call(input)?;
        self.second.call(output)
    }
}

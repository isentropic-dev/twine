use crate::Component;

/// A component that sequentially composes two components.
///
/// `Then` enforces that the first component's output type matches the
/// second component's input type, ensuring type safety in chained execution.
///
/// When `call()` is invoked, the input is passed to the first component (`A`),
/// and its output is forwarded as the input to the second component (`B`).
///
/// This guarantees that `A::Output` is always compatible with `B::Input`,
/// making it safe to compose arbitrary components as long as their types align.
pub(crate) struct Then<A, B> {
    first: A,
    second: B,
}

impl<A, B> Then<A, B> {
    /// Creates a new sequential composition of two components.
    pub(crate) const fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A, B> Component for Then<A, B>
where
    A: Component,
    B: Component<Input = A::Output>,
{
    type Input = A::Input;
    type Output = B::Output;

    fn call(&self, input: Self::Input) -> Self::Output {
        let output = self.first.call(input);
        self.second.call(output)
    }
}

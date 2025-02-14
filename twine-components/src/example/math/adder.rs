use std::ops::Add;

use twine_core::Component;

/// A component that adds a constant increment to an input value.
///
/// Given an input of type `T`, this component adds a predefined increment and
/// produces an output of the same type.
///
/// # Type Parameters
///
/// - `T`: The input and output type, which must support addition.
pub struct Adder<T> {
    increment: T,
}

impl<T> Adder<T>
where
    T: Add<Output = T> + Copy,
{
    /// Creates a new [`Adder`] with the specified increment.
    ///
    /// # Parameters
    ///
    /// - `increment`: The value to be added to each input.
    pub fn new(increment: T) -> Self {
        Self { increment }
    }
}

impl<T> Component for Adder<T>
where
    T: Add<Output = T> + Copy,
{
    type Input = T;
    type Output = T;

    fn call(&self, input: Self::Input) -> Self::Output {
        input + self.increment
    }
}

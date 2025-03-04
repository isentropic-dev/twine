use std::{convert::Infallible, ops::Mul};

use twine_core::Component;

/// A component that multiplies an input value by a constant factor.
///
/// Given an input of type `T`, this component multiplies it by a predefined
/// factor and produces an output of the same type.
///
/// # Type Parameters
///
/// - `T`: The input and output type, which must support multiplication.
pub struct Multiplier<T> {
    factor: T,
}

impl<T> Multiplier<T>
where
    T: Mul<Output = T> + Copy,
{
    /// Creates a new [`Multiplier`] with the specified factor.
    ///
    /// # Parameters
    ///
    /// - `factor`: The value by which each input will be multiplied.
    pub fn new(factor: T) -> Self {
        Self { factor }
    }
}

impl<T> Component for Multiplier<T>
where
    T: Mul<Output = T> + Copy,
{
    type Input = T;
    type Output = T;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(input * self.factor)
    }
}

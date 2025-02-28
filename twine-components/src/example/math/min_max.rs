use std::convert::Infallible;

use twine_core::Component;

/// A component that computes the minimum and maximum of two `f64` values.
pub struct MinMax;

/// The input for the [`MinMax`] component.
pub struct MinMaxInput {
    pub x: f64,
    pub y: f64,
}

/// The output for the [`MinMax`] component.
pub struct MinMaxOutput {
    pub min: f64,
    pub max: f64,
}

impl Component for MinMax {
    type Input = MinMaxInput;
    type Output = MinMaxOutput;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(Self::Output {
            min: input.x.min(input.y),
            max: input.x.max(input.y),
        })
    }
}

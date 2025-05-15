use std::convert::Infallible;

use uom::si::f64::Time;

use crate::Component;

/// A test component that returns its input time unchanged.
///
/// Useful for verifying time progression and simulation mechanics.
#[derive(Debug)]
pub(crate) struct EchoTime;

impl Component for EchoTime {
    type Input = Time;
    type Output = Time;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(input)
    }
}

use std::convert::Infallible;

use crate::{
    transient::{Controller, Simulation, Temporal},
    Component,
};

/// A no-op [`Controller`] that passes inputs through unchanged.
///
/// Use `NoController` when control logic is unnecessary.
/// It returns the input unchanged, making it ideal for open-loop simulations or
/// for systems driven entirely by the integrator.
#[derive(Debug)]
pub struct NoController;

impl<C> Controller<C> for NoController
where
    C: Component,
    C::Input: Clone + Temporal,
{
    type Error = Infallible;

    fn adjust_input(
        &self,
        _simulation: &Simulation<C>,
        input: C::Input,
    ) -> Result<C::Input, Self::Error> {
        Ok(input)
    }
}

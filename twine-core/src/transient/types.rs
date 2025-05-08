use std::{fmt::Debug, ops::Div};

use uom::si::f64::Time;

use crate::Component;

use super::Temporal;

/// A snapshot of a [`Component`]’s behavior at a single simulation step.
///
/// A `TimeStep` stores the `input` and corresponding `output` of a component at
/// a specific point in simulated time.
/// It represents an atomic step in the simulation’s progression and serves as
/// the core unit for integration and history tracking.
///
/// Trait implementations like `Copy`, `PartialEq`, and `Ord` are derived
/// conditionally based on the component's `Input` and `Output` types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TimeStep<C>
where
    C: Component,
    C::Input: Temporal,
{
    pub input: C::Input,
    pub output: C::Output,
}

impl<C> TimeStep<C>
where
    C: Component,
    C::Input: Temporal,
{
    /// Creates a new [`TimeStep`] from the given input and output.
    pub fn new(input: C::Input, output: C::Output) -> Self {
        Self { input, output }
    }
}

/// The type resulting from dividing `T` by [`Time`].
///
/// Commonly used to represent the time derivative of a physical quantity.
/// For example, `TimeDerivativeOf<Length>` is `Velocity`.
pub type TimeDerivativeOf<T> = <T as Div<Time>>::Output;

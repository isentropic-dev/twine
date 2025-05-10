pub mod integrators;
mod time_derivative;
mod traits;
mod transient_simulation;

#[cfg(test)]
mod test_utils;

use std::fmt::Debug;

pub use time_derivative::{HasTimeDerivative, TimeDerivativeOf};
pub use traits::{HasTime, StateIntegrator, StatefulComponent};
pub use transient_simulation::TransientSimulation;

/// A snapshot of a [`StatefulComponent`]’s result at a single simulation step.
///
/// A `TimeStep` records the `input` and corresponding `output` of a component
/// at a specific point in simulated time.
/// It represents an atomic step in the transient simulation’s progression and
/// acts as a fundamental unit for history tracking, stepping, and integration.
#[derive(Clone, Debug)]
pub struct TimeStep<C>
where
    C: StatefulComponent,
    C::Input: Clone + Debug,
    C::Output: Clone + Debug,
{
    pub input: C::Input,
    pub output: C::Output,
}

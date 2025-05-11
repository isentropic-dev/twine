use std::fmt::Debug;

use uom::si::f64::Time;

use crate::Component;

use super::{HasTimeDerivative, TimeStep};

/// Trait for extracting and updating simulation time within a type.
///
/// Typically implemented on a component’s `Input` type, this trait enables
/// time-aware operations by exposing an embedded [`uom::si::f64::Time`] value.
pub trait HasTime: Sized {
    /// Returns the current simulation time.
    fn get_time(&self) -> Time;

    /// Returns `self` with its time set to the given value.
    #[must_use]
    fn with_time(self, time: Time) -> Self;
}

/// Extension trait for [`Component`]s that evolve state over time.
///
/// A `StatefulComponent` is one whose internal state changes according to time
/// derivatives that it computes internally and includes in its output.
///
/// This trait defines how to extract state variables from the component’s
/// input, state derivatives from its output, and construct a new input
/// reflecting an updated state.
///
/// It is typically used in conjunction with a [`StateIntegrator`] to step the
/// component forward in time.
pub trait StatefulComponent: Component {
    /// Represents the state variables in the component.
    type State: HasTimeDerivative;

    /// Extracts the current state from a given input.
    fn extract_state(input: &Self::Input) -> Self::State;

    /// Extracts the time derivative of the state from a given output.
    fn extract_derivative(
        output: &Self::Output,
    ) -> <Self::State as HasTimeDerivative>::TimeDerivative;

    /// Applies a new state to an existing input, producing the next input.
    fn apply_state(input: &Self::Input, state: Self::State) -> Self::Input;
}

/// An integrator for integrating a [`StatefulComponent`] forward in time.
///
/// A `StateIntegrator` evolves a component’s internal state by integrating its
/// time derivative over a discrete interval. Given a current [`TimeStep`] and a
/// time increment, it produces the next [`Component::Input`].
///
/// This trait provides a reusable interface for implementing different
/// integration schemes such as Euler or Runge-Kutta.
pub trait StateIntegrator<C>
where
    C: StatefulComponent,
    C::Input: Clone + Debug + HasTime,
    C::Output: Clone + Debug,
{
    /// Integrates the component's state one step forward in time.
    ///
    /// # Errors
    ///
    /// Returns `Err(C::Error)` if the component fails during evaluation.
    fn integrate_state(
        &self,
        component: &C,
        current: &TimeStep<C>,
        dt: Time,
    ) -> Result<C::Input, C::Error>;
}

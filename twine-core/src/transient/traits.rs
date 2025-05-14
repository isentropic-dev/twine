use std::ops::Div;

use uom::si::f64::Time;

use crate::Component;

use super::TimeStep;

/// A trait for types that carry and expose simulation time.
///
/// `Temporal` is typically implemented on a component’s input type to support
/// time-aware operations in simulations.
/// It provides access to the current simulation time and a way to produce a new
/// instance with a modified time value.
pub trait Temporal: Sized {
    /// Returns the current simulation time.
    fn get_time(&self) -> Time;

    /// Consumes `self` and returns a new instance with the given time.
    #[must_use]
    fn with_time(self, time: Time) -> Self;
}

/// A trait for indicating that a type has a time derivative.
///
/// This trait defines the type that represents the rate of change of `Self`
/// with respect to time, which is used in simulations and integration logic.
pub trait HasTimeDerivative {
    type TimeDerivative;
}

/// A trait for proposing the next input in a simulation.
///
/// An `Integrator` generates a new input for a [`Component`] using the
/// simulation history and a time increment.
/// The proposed input reflects the component’s expected state at the next step,
/// but may be refined by a [`Controller`] before evaluation.
///
/// Integrators implement time-stepping methods like forward Euler, Runge-Kutta,
/// and other schemes.
pub trait Integrator<C>
where
    C: Component,
    C::Input: Temporal,
{
    type Error: std::error::Error + Send + Sync + 'static;

    /// Proposes the next input for the component.
    ///
    /// Computes a candidate input that predicts the system state after
    /// advancing by `dt`, based on the component's behavior and history.
    ///
    /// The integration strategy (e.g., forward Euler, Runge-Kutta) defines how
    /// the input is computed.
    /// This input may be used directly or refined by a [`Controller`] before it
    /// is evaluated.
    ///
    /// # Parameters
    ///
    /// - `component`: The component being simulated.
    /// - `history`: Prior [`TimeStep`]s in chronological order.
    /// - `dt`: Time step to advance the simulation.
    ///
    /// # Returns
    ///
    /// A proposed input for the next simulation step.
    ///
    /// # Errors
    ///
    /// Returns `Err(Self::Error)` if integration fails.
    fn propose_input(
        &self,
        component: &C,
        history: &[TimeStep<C>],
        dt: Time,
    ) -> Result<C::Input, Self::Error>;
}

/// A trait for adjusting an input before it is evaluated by a component.
///
/// A `Controller` modifies inputs proposed by an [`Integrator`] by applying
/// control logic based on the component and its simulation history.
///
/// Controllers are optional but useful when inputs depend on system state,
/// outputs, or target behavior.
pub trait Controller<C>
where
    C: Component,
    C::Input: Temporal,
{
    type Error: std::error::Error + Send + Sync + 'static;

    /// Adjusts the input proposed by an integrator before evaluation.
    ///
    /// A controller can modify the input based on the component and its
    /// simulation history.
    /// This is useful for feedback control, constraint enforcement, or other
    /// context-specific logic.
    ///
    /// # Errors
    ///
    /// Returns `Err(Self::Error)` if adjustment fails.
    fn adjust_input(
        &self,
        component: &C,
        history: &[TimeStep<C>],
        input: C::Input,
    ) -> Result<C::Input, Self::Error>;
}

/// A trait for components that evolve internal state over time.
///
/// A `StatefulComponent` is a [`Component`] whose input includes state
/// variables and whose output encodes their time derivatives, allowing
/// integration over time.
/// This trait defines how to extract the state from an input, extract its time
/// derivative from an output, and apply a new state to generate the next input.
///
/// It allows integrators to evolve the system state by applying time-scaled
/// derivatives across simulation steps.
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

/// Blanket implementation for types that support division by time.
///
/// Any type that implements `Div<Time>` automatically satisfies
/// [`HasTimeDerivative`], with the division result used as its time derivative.
///
/// This implementation allows most `uom::si::Quantity` types to participate in
/// time-based simulation without additional boilerplate.
impl<T> HasTimeDerivative for T
where
    T: Div<Time>,
{
    type TimeDerivative = T::Output;
}

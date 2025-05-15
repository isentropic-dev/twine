use crate::{transient::HasTimeDerivative, Component};

/// A trait for components that evolve internal state over time.
///
/// A `StatefulComponent` is a [`Component`] whose input encodes system state,
/// and whose output provides the corresponding time derivatives.
/// This design enables integration over time by clearly separating state
/// extraction, derivative evaluation, and state reapplication.
pub trait StatefulComponent: Component {
    /// Represents the state variables in the component.
    type State: HasTimeDerivative;

    /// Extracts the internal state from a given input.
    ///
    /// This method is called by integrators to read the current state variables
    /// from the component's input before advancing the simulation.
    fn extract_state(input: &Self::Input) -> Self::State;

    /// Extracts the time derivative of the state from a given output.
    ///
    /// This method returns the rate of change corresponding to the state
    /// extracted by [`extract_state`].
    /// It must match the ordering and semantics of that state.
    fn extract_derivative(
        output: &Self::Output,
    ) -> <Self::State as HasTimeDerivative>::TimeDerivative;

    /// Applies a new state to an existing input, producing the next input.
    ///
    /// This method injects the updated state back into a previous input,
    /// typically after the integrator computes a new state by applying
    /// time-scaled derivatives.
    fn apply_state(input: &Self::Input, state: Self::State) -> Self::Input;
}

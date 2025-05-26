use std::ops::{Add, Div, Mul};

use uom::si::f64::Time;

use crate::{transient::TimeDerivativeOf, Component};

/// A trait for components with time-evolving internal state.
///
/// A `StatefulComponent` is a specialized [`Component`] whose input encodes
/// dynamic system state, and whose output provides the corresponding time
/// derivatives.
/// This design enables integration over time by clearly separating state
/// extraction, derivative evaluation, and state reapplication.
///
/// # Integration Semantics
///
/// This trait defines three core operations:
///
/// 1. [`extract_state`] retrieves the current state from the component's input.
/// 2. [`extract_derivative`] computes the time derivative of that state from the output.
/// 3. [`apply_state`] injects a new state into a previous input to produce the next input.
///
/// Together, these operations provide the minimal interface required for
/// time-based integration using external stepping logic.
///
/// # Arithmetic Requirements on `State`
///
/// The associated `State` type must support:
///
/// - Division by [`Time`] to yield a time derivative.
/// - Multiplication of that derivative by [`Time`] to compute a delta.
/// - Addition of the delta to the original state to produce a new state.
///
/// These constraints are automatically satisfied by most physical quantity
/// types from the [`uom`] crate.
/// If you are using custom types, ensure they implement the necessary
/// arithmetic traits.
pub trait StatefulComponent: Component
where
    Self::State: Div<Time>,
    TimeDerivativeOf<Self::State>: Mul<Time>,
    Self::State: Add<StateDelta<Self::State>, Output = Self::State>,
{
    /// The type representing the component's time-evolving internal state.
    ///
    /// This type may be a scalar, vector, tuple, or custom struct, depending on
    /// the system being modeled.
    /// These values are subject to integration and updated as the simulation progresses.
    type State;

    /// Extracts the current state from the component's input.
    ///
    /// Called by an integrator to obtain the state at the current time.
    fn extract_state(input: &Self::Input) -> Self::State;

    /// Extracts the time derivative of the state from the component's output.
    ///
    /// This value represents the rate of change of the state at the current time.
    fn extract_derivative(output: &Self::Output) -> TimeDerivativeOf<Self::State>;

    /// Applies an updated state to a previous input, producing the next input.
    ///
    /// This method is called after an integrator has evolved the component's state.
    /// It returns a new input with the updated state injected, while preserving
    /// all other parts of the original input.
    ///
    /// The result represents the component's next input at a new point in time
    /// using the evolved state.
    fn apply_state(input: &Self::Input, state: Self::State) -> Self::Input;
}

/// Internal alias representing a finite change in state over time.
///
/// This type is computed by multiplying a time derivative by a duration.
/// It simplifies trait bounds within [`StatefulComponent`] and is not intended
/// for public use.
type StateDelta<T> = <TimeDerivativeOf<T> as Mul<Time>>::Output;

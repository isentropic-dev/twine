use crate::{
    transient::{Simulation, Temporal, TimeIncrement},
    Component,
};

/// A trait for proposing the next input in a simulation step.
///
/// An `Integrator` generates a new input for a [`Component`] using the current
/// simulation history and a time increment.
/// The proposed input estimates the system state at the next point in time and
/// is typically passed to a [`Controller`] before evaluation.
///
/// This trait enables the use of custom integration schemes such as forward
/// Euler, Runge-Kutta, or multistep methods.
///
/// # Example Implementations
///
/// - [`AdvanceTime`](crate::transient::integrators::AdvanceTime): time-only progression.
/// - [`ForwardEuler`](crate::transient::integrators::ForwardEuler): a first-order integrator.
///
/// # Usage
///
/// Implement this trait to define how the simulation advances from one
/// time step to the next, based on the component’s behavior and the current
/// simulation history.
pub trait Integrator<C>
where
    C: Component,
    C::Input: Temporal,
{
    /// The error type returned if integration fails.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Proposes the next component input based on the current simulation state.
    ///
    /// This method computes a candidate input that advances the simulation by
    /// the given time increment `dt`.
    /// The result reflects the estimated state at the next simulation step and
    /// may be adjusted by a [`Controller`] before evaluation.
    ///
    /// # Parameters
    ///
    /// - `simulation`: The current simulation, including input/output history.
    /// - `dt`: The time step to advance by.
    ///
    /// # Returns
    ///
    /// A proposed input for the component at the next simulation step.
    ///
    /// # Errors
    ///
    /// Returns `Err(Self::Error)` if integration fails.
    fn propose_input(
        &self,
        simulation: &Simulation<C>,
        dt: TimeIncrement,
    ) -> Result<C::Input, Self::Error>;
}

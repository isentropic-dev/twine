use std::{convert::Infallible, time::Duration};

/// Defines a scheme for advancing state variables over time.
///
/// An `Integrator` evolves part of a system forward using a specific method,
/// such as Forward Euler, Runge-Kutta, or a symbolic or custom strategy.
/// It operates on values with known time derivatives and is agnostic to the
/// surrounding simulation or model structure.
///
/// Integrators may use fixed or adaptive time stepping.
pub trait Integrator {
    /// The input required to perform integration.
    ///
    /// Typically includes state variables with time derivatives (e.g., position
    /// and velocity), plus any configuration needed by the integrator.
    /// Implementations may also accept closures for deferred evaluation, such
    /// as computing intermediate derivatives during multi-step methods.
    type Input;

    /// The output produced by integration.
    ///
    /// Usually contains the updated values of integrated variables after one
    /// step forward in time.
    type Output;

    /// The error type returned if integration fails.
    ///
    /// May represent instability, invalid input, or failure of adaptive logic.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Performs integration over a single time step.
    ///
    /// Applies the integration method to the given input using the requested `dt`,
    /// and returns the updated output along with the actual time step taken.
    /// Adaptive integrators may adjust the time step based on internal logic.
    ///
    /// # Parameters
    ///
    /// - `input`: The data to integrate.
    /// - `dt`: The proposed time step duration.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The integration output.
    /// - The actual time step taken (which may differ from `dt`).
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the integration step fails.
    fn integrate(
        &self,
        input: Self::Input,
        dt: Duration,
    ) -> Result<(Self::Output, Duration), Self::Error>;
}

/// A no-op integrator that always succeeds.
///
/// The `()` integrator is useful for stateless or discrete-time simulations,
/// or for testing when integration is unnecessary.
impl Integrator for () {
    type Input = ();
    type Output = ();
    type Error = Infallible;

    fn integrate(
        &self,
        _input: Self::Input,
        dt: Duration,
    ) -> Result<(Self::Output, Duration), Self::Error> {
        Ok(((), dt))
    }
}

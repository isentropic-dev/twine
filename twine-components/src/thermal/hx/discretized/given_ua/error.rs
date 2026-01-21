use thiserror::Error;
use twine_solve::equation::bisection;
use uom::si::f64::ThermalConductance;

use crate::thermal::hx::discretized::SolveError;

/// Errors that can occur while solving given a target conductance (UA).
#[derive(Debug, Error)]
pub enum GivenUaError {
    /// A discretized heat exchanger solve failed.
    #[error("discretized solve failed")]
    Solve(#[from] SolveError),

    /// The bisection solver encountered an error.
    #[error("bisection solver error")]
    Bisection(#[from] bisection::Error),

    /// The solver reached the iteration limit without converging.
    #[error("solver hit iteration limit: residual={residual:?}")]
    MaxIters {
        /// Best UA residual achieved.
        ///
        /// This is the smallest absolute difference between achieved and target
        /// conductance encountered during iteration.
        residual: ThermalConductance,

        /// Iteration count performed by the solver.
        iters: usize,
    },
}

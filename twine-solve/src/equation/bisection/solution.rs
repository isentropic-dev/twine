use crate::model::Snapshot;

/// Indicates whether the solver converged or hit the iteration limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Converged according to the configured tolerances.
    Converged,
    /// Reached the iteration limit without converging.
    MaxIters,
    /// Stopped early due to an observer decision.
    StoppedByObserver,
}

/// The result of a bisection solve.
#[derive(Debug, Clone)]
pub struct Solution<I, O> {
    /// Final solver status.
    pub status: Status,
    /// Best estimate of the root.
    pub x: f64,
    /// Residual at the reported root estimate.
    pub residual: f64,
    /// Snapshot at the reported root estimate.
    pub snapshot: Snapshot<I, O>,
    /// Iteration count when the solver finished.
    pub iters: usize,
}

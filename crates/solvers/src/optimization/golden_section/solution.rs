use twine_core::Snapshot;

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

/// The result of a golden section search.
#[derive(Debug, Clone)]
pub struct Solution<I, O> {
    /// Final solver status.
    pub status: Status,

    /// Best estimate of the optimum x.
    pub x: f64,

    /// Objective value at the reported x.
    pub objective: f64,

    /// Snapshot at the reported x.
    pub snapshot: Snapshot<I, O>,

    /// Iteration count when the solver finished.
    pub iters: usize,
}

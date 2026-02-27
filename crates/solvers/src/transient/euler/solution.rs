use twine_core::Snapshot;

/// Indicates how the solver terminated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Completed all requested steps.
    Complete,

    /// Stopped early due to an observer action.
    StoppedByObserver,
}

/// The result of an Euler integration.
#[derive(Debug, Clone)]
pub struct Solution<I, O> {
    /// How the solver terminated.
    pub status: Status,

    /// History of snapshots from each step (including initial state).
    pub history: Vec<Snapshot<I, O>>,

    /// Number of integration steps completed.
    pub steps: usize,
}

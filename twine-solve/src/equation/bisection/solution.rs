use crate::{equation::Evaluation, model::Snapshot};

/// Indicates whether the solver converged or hit the iteration limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Converged,
    MaxIters,
}

/// The result of a bisection solve.
#[derive(Debug, Clone)]
pub struct Solution<I, O> {
    pub status: Status,
    pub x: f64,
    pub residual: f64,
    pub snapshot: Snapshot<I, O>,
    pub iters: usize,
}

impl<I, O> Solution<I, O> {
    /// Constructs a solution from an evaluation result.
    pub(super) fn from_eval(eval: Evaluation<I, O, 1>, status: Status, iters: usize) -> Self {
        Self {
            status,
            x: eval.x[0],
            residual: eval.residuals[0],
            snapshot: eval.snapshot,
            iters,
        }
    }
}

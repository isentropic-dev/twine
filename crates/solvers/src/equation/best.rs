use crate::equation::Evaluation;

use super::solution::{Solution, Status};

/// Tracks the best evaluation encountered so far.
///
/// The best evaluation is defined by minimum residual magnitude.
/// The bracket can shrink without any successful evaluation (via observer
/// recovery), so `None` is a normal operating state — not an error condition.
pub(crate) struct Best<I, O> {
    eval: Option<Evaluation<I, O, 1>>,
}

impl<I, O> Best<I, O> {
    /// Creates an empty best tracker.
    pub(crate) fn empty() -> Self {
        Self { eval: None }
    }

    /// Updates the best evaluation if the residual magnitude improves.
    pub(crate) fn update(&mut self, eval: Evaluation<I, O, 1>) {
        if let Some(best) = self.eval.as_ref()
            && eval.residuals[0].abs() >= best.residuals[0].abs()
        {
            return;
        }
        self.eval = Some(eval);
    }

    /// Returns true if the best residual meets the tolerance.
    pub(crate) fn is_residual_converged(&self, residual_tol: f64) -> bool {
        self.eval
            .as_ref()
            .is_some_and(|eval| eval.residuals[0].abs() <= residual_tol)
    }

    /// Builds a solution from the best evaluation.
    ///
    /// Returns `None` if no successful evaluation has been recorded.
    pub(crate) fn into_solution(self, status: Status, iters: usize) -> Option<Solution<I, O>> {
        let eval = self.eval?;
        Some(Solution {
            status,
            x: eval.x[0],
            residual: eval.residuals[0],
            snapshot: eval.snapshot,
            iters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    use twine_core::Snapshot;

    fn eval(x: f64, residual: f64) -> Evaluation<(), (), 1> {
        Evaluation {
            x: [x],
            residuals: [residual],
            snapshot: Snapshot::new((), ()),
        }
    }

    #[test]
    fn update_keeps_best_residual() {
        let mut best = Best::empty();
        best.update(eval(1.0, 2.0));
        best.update(eval(2.0, -1.5));
        best.update(eval(3.0, 1.0));

        let solution = best
            .into_solution(Status::StoppedByObserver, 0)
            .expect("best eval");

        assert_relative_eq!(solution.x, 3.0);
        assert_relative_eq!(solution.residual, 1.0);
    }

    #[test]
    fn update_ignores_worse_residual() {
        let mut best = Best::empty();
        best.update(eval(1.0, -0.5));
        best.update(eval(2.0, 2.0));

        let solution = best
            .into_solution(Status::StoppedByObserver, 0)
            .expect("best eval");

        assert_relative_eq!(solution.x, 1.0);
        assert_relative_eq!(solution.residual, -0.5);
    }

    #[test]
    fn residual_converged_requires_eval() {
        let best: Best<(), ()> = Best::empty();
        assert!(!best.is_residual_converged(1e-3));
    }

    #[test]
    fn residual_converged_checks_tolerance() {
        let mut best = Best::empty();
        best.update(eval(1.0, 1e-2));

        assert!(!best.is_residual_converged(1e-3));
        assert!(best.is_residual_converged(1e-1));
    }

    #[test]
    fn into_solution_returns_none_without_eval() {
        let best: Best<(), ()> = Best::empty();
        assert!(best.into_solution(Status::StoppedByObserver, 0).is_none());
    }

    #[test]
    fn into_solution_builds_solution() {
        let mut best = Best::empty();
        best.update(eval(2.0, -1.25));

        let solution = best.into_solution(Status::Converged, 4).expect("best eval");

        assert_eq!(solution.status, Status::Converged);
        assert_eq!(solution.iters, 4);
        assert_relative_eq!(solution.x, 2.0);
        assert_relative_eq!(solution.residual, -1.25);
    }
}

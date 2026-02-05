use crate::equation::Evaluation;

use super::{Error, Solution, Status};

/// Tracks the best evaluation encountered so far.
///
/// The best evaluation is defined by minimum residual magnitude.
/// The `Option` lets us represent the state before any successful evaluation.
pub(super) struct Best<I, O> {
    eval: Option<Evaluation<I, O, 1>>,
}

impl<I, O> Best<I, O> {
    /// Creates an empty best tracker.
    pub(super) fn empty() -> Self {
        Self { eval: None }
    }

    /// Updates the best evaluation if the residual magnitude improves.
    pub(super) fn update(&mut self, eval: Evaluation<I, O, 1>) {
        if let Some(best) = self.eval.as_ref()
            && eval.residuals[0].abs() >= best.residuals[0].abs()
        {
            return;
        }
        self.eval = Some(eval);
    }

    /// Returns true if the best residual meets the tolerance.
    pub(super) fn is_residual_converged(&self, residual_tol: f64) -> bool {
        self.eval
            .as_ref()
            .is_some_and(|eval| eval.residuals[0].abs() <= residual_tol)
    }

    /// Finalizes the solver using the best available evaluation.
    ///
    /// # Errors
    ///
    /// Returns `Error::NoSuccessfulEvaluation` if no successful evaluation is stored.
    pub(super) fn finish(self, status: Status, iters: usize) -> Result<Solution<I, O>, Error> {
        let eval = self.eval.ok_or(Error::NoSuccessfulEvaluation)?;
        Ok(Solution {
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
            .finish(Status::StoppedByObserver, 0)
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
            .finish(Status::StoppedByObserver, 0)
            .expect("best eval");

        assert_relative_eq!(solution.x, 1.0);
        assert_relative_eq!(solution.residual, -0.5);
    }

    #[test]
    fn residual_converged_requires_best() {
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
    fn finish_errors_without_eval() {
        let best: Best<(), ()> = Best::empty();
        let err = best.finish(Status::StoppedByObserver, 0);
        assert!(matches!(err, Err(Error::NoSuccessfulEvaluation)));
    }

    #[test]
    fn finish_builds_solution() {
        let mut best = Best::empty();
        best.update(eval(2.0, -1.25));

        let solution = best.finish(Status::Converged, 4).expect("best eval");

        assert_eq!(solution.status, Status::Converged);
        assert_eq!(solution.iters, 4);
        assert_relative_eq!(solution.x, 2.0);
        assert_relative_eq!(solution.residual, -1.25);
    }
}

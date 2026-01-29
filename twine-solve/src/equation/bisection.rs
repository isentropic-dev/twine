mod action;
mod best;
mod bracket;
mod config;
mod decision;
mod error;
mod eval_context;
mod event;
mod solution;

pub use action::Action;
pub use bracket::{Bracket, BracketError, Sign};
pub use config::{Config, ConfigError};
pub use error::Error;
pub use event::Event;
pub use solution::{Solution, Status};

use twine_core::model::Model;

use crate::Observer;

use super::EquationProblem;

use best::Best;
use bracket::Bounds;
use decision::Decision;
use eval_context::EvalContext;

/// Finds a root of the equation using the bisection method.
///
/// # Algorithm
///
/// 1. Evaluate the left and right endpoints.
/// 2. Validate that the endpoints bracket a root using residual signs.
/// 3. Iterate: evaluate the midpoint, shrink the bracket, and update the best evaluation.
///
/// Convergence is reported when either:
/// - The best residual magnitude is within `config.residual_tol` (absolute only), or
/// - The bracket width satisfies `x_abs_tol + x_rel_tol * |mid|`.
///
/// # Observer
///
/// The observer receives an [`Event`] for each evaluation and may:
/// - Return `Action::StopEarly` to stop and return the best evaluation so far.
/// - Return `Action::AssumeResidualSign(Sign)` to recover from evaluation
///   failures by providing a residual sign for bracket updates.
///   When this action is used on a successful evaluation, that evaluation is
///   not considered for the best solution.
///
/// # Notes
///
/// The returned [`Solution`] always reflects the best successful evaluation
/// seen so far (by residual magnitude).
/// Iteration counts correspond to the number of midpoint evaluations performed.
///
/// # Errors
///
/// Returns an error if the bracket is invalid, the config is invalid,
/// or the model or problem returns an unrecovered error during evaluation.
pub fn solve<M, P, Obs>(
    model: &M,
    problem: &P,
    bracket: [f64; 2],
    config: &Config,
    mut observer: Obs,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    M::Input: Clone,
    M::Output: Clone,
    P: EquationProblem<1, Input = M::Input, Output = M::Output>,
    Obs: for<'a> Observer<Event<'a, M, P>, Action>,
{
    config.validate()?;

    // Validate and order initial bounds.
    let bounds = Bounds::new(bracket)?;
    let [left, right] = bounds.as_array();

    let mut best = Best::empty();
    let mut ctx = EvalContext::new(model, problem, &mut observer);

    // Resolve left endpoint.
    let (left_eval, left_decision) = ctx.left_endpoint(left);
    if let Some(eval) = left_eval {
        best.update(eval);
    }
    let left_sign = match left_decision {
        Decision::Continue(sign) => sign,
        Decision::StopEarly => return best.finish(Status::StoppedByObserver, 0),
        Decision::Error(error) => return Err(error),
    };

    // Resolve right endpoint.
    let (right_eval, right_decision) = ctx.right_endpoint(right);
    if let Some(eval) = right_eval {
        best.update(eval);
    }
    let right_sign = match right_decision {
        Decision::Continue(sign) => sign,
        Decision::StopEarly => return best.finish(Status::StoppedByObserver, 0),
        Decision::Error(error) => return Err(error),
    };

    // Validate bracket signs now that both endpoints are known.
    let mut bracket = Bracket::new(bounds, left_sign, right_sign)?;

    if best.is_residual_converged(config.residual_tol) {
        return best.finish(Status::Converged, 0);
    }

    // Iterate by shrinking the bracket with midpoint evaluations.
    for iter in 1..=config.max_iters {
        if bracket.is_x_converged(config.x_abs_tol, config.x_rel_tol) {
            return best.finish(Status::Converged, iter - 1);
        }

        // Evaluate the midpoint and update the bracket.
        let mid = bracket.midpoint();
        let (mid_eval, mid_decision) = ctx.midpoint(mid, &bracket);
        if let Some(eval) = mid_eval {
            best.update(eval);
        }
        match mid_decision {
            Decision::Continue(sign) => bracket.shrink(mid, sign),
            Decision::StopEarly => {
                return best.finish(Status::StoppedByObserver, iter);
            }
            Decision::Error(error) => return Err(error),
        }

        if best.is_residual_converged(config.residual_tol) {
            return best.finish(Status::Converged, iter);
        }
    }

    best.finish(Status::MaxIters, config.max_iters)
}

/// Runs bisection without observation.
///
/// # Errors
///
/// Returns an error if the bracket is invalid, the config is invalid,
/// or the model or problem returns an error during evaluation.
pub fn solve_unobserved<M, P>(
    model: &M,
    problem: &P,
    bracket: [f64; 2],
    config: &Config,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    M::Input: Clone,
    M::Output: Clone,
    P: EquationProblem<1, Input = M::Input, Output = M::Output>,
{
    solve(model, problem, bracket, config, ())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::Infallible;

    use approx::assert_relative_eq;
    use thiserror::Error;

    /// Model that squares its input.
    struct SquareModel;
    impl Model for SquareModel {
        type Input = f64;
        type Output = f64;
        type Error = Infallible;

        fn call(&self, input: &Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(input * input)
        }
    }

    /// Model that cubes its input.
    struct CubeModel;
    impl Model for CubeModel {
        type Input = f64;
        type Output = f64;
        type Error = Infallible;

        fn call(&self, input: &Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(input * input * input)
        }
    }

    /// Model that fails above a threshold (like HX second law violations).
    struct ThresholdModel {
        threshold: f64,
    }
    #[derive(Debug, Clone, Error)]
    #[error("exceeded threshold at x={x}")]
    struct ThresholdError {
        x: f64,
    }
    impl Model for ThresholdModel {
        type Input = f64;
        type Output = f64;
        type Error = ThresholdError;

        fn call(&self, input: &Self::Input) -> Result<Self::Output, Self::Error> {
            if *input > self.threshold {
                Err(ThresholdError { x: *input })
            } else {
                Ok(input * input)
            }
        }
    }

    /// Equation problem that drives the model output to a target value.
    /// Residual is `output - target` for any f64→f64 model.
    struct TargetOutputProblem {
        target: f64,
    }
    impl EquationProblem<1> for TargetOutputProblem {
        type Input = f64;
        type Output = f64;
        type InputError = Infallible;
        type ResidualError = Infallible;

        fn input(&self, x: &[f64; 1]) -> Result<Self::Input, Self::InputError> {
            Ok(x[0])
        }

        fn residuals(
            &self,
            _input: &Self::Input,
            output: &Self::Output,
        ) -> Result<[f64; 1], Self::ResidualError> {
            Ok([output - self.target])
        }
    }

    #[test]
    fn finds_square_root() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 9.0 };

        let solution = solve_unobserved(&model, &problem, [0.0, 10.0], &Config::default())
            .expect("should solve");

        assert_eq!(solution.status, Status::Converged);
        assert_relative_eq!(solution.x, 3.0, epsilon = 1e-10);
        assert_relative_eq!(solution.snapshot.output, 9.0, epsilon = 1e-10);
    }

    #[test]
    fn finds_cube_root() {
        let model = CubeModel;
        let problem = TargetOutputProblem { target: 27.0 };

        let solution = solve_unobserved(&model, &problem, [0.0, 10.0], &Config::default())
            .expect("should solve");

        assert_eq!(solution.status, Status::Converged);
        assert_relative_eq!(solution.x, 3.0, epsilon = 1e-10);
        assert_relative_eq!(solution.snapshot.output, 27.0, epsilon = 1e-10);
    }

    #[test]
    fn observer_can_stop_iteration() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 9.0 };

        let mut midpoint_count = 0usize;
        let observer = |event: &Event<'_, _, _>| {
            if matches!(event, Event::Midpoint { .. }) {
                midpoint_count += 1;
                if midpoint_count >= 3 {
                    return Some(Action::StopEarly);
                }
            }
            None
        };

        let solution = solve(&model, &problem, [0.0, 10.0], &Config::default(), observer)
            .expect("should stop cleanly");

        assert_eq!(solution.status, Status::StoppedByObserver);
        assert_eq!(solution.iters, 3);
        assert_eq!(midpoint_count, 3);
    }

    #[test]
    fn zero_iters_returns_best_endpoint() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 9.0 };

        let config = Config {
            max_iters: 0,
            ..Config::default()
        };
        let solution = solve_unobserved(&model, &problem, [2.0, 10.0], &config)
            .expect("should return best endpoint");

        assert_eq!(solution.status, Status::MaxIters);
        assert_eq!(solution.iters, 0);
        // x=2 gives residual |4-9|=5, x=10 gives |100-9|=91
        // So best endpoint should be x=2
        assert_relative_eq!(solution.x, 2.0);
    }

    #[test]
    fn observer_can_recover_from_eval_failure() {
        // Model fails above x=7, root is at x=3 (for target=9)
        let model = ThresholdModel { threshold: 7.0 };
        let problem = TargetOutputProblem { target: 9.0 };

        // Initial bracket [0, 10] would fail at right endpoint (x=10 > threshold=7)
        // Observer tells solver to use a positive residual for failed points
        // (points above threshold would have large positive residuals: x^2 - 9 > 0)
        let observer = |event: &Event<'_, _, _>| {
            let is_err = event.result().is_err();
            if is_err {
                // Failed points are above threshold, so residual would be positive
                Some(Action::assume_positive())
            } else {
                None
            }
        };

        let solution = solve(&model, &problem, [0.0, 10.0], &Config::default(), observer)
            .expect("should recover and solve");

        assert_eq!(solution.status, Status::Converged);
        assert_relative_eq!(solution.x, 3.0, epsilon = 1e-10);
    }

    #[test]
    fn midpoint_failure_assumes_sign() {
        // Model fails above x=3.5, root is at x=3 (for target=9)
        // Initial bracket [0, 3.5] is valid, midpoint=1.75 is valid
        // But as bisection homes in from the left, midpoints > 3.5 will fail
        let model = ThresholdModel { threshold: 3.5 };
        let problem = TargetOutputProblem { target: 9.0 };

        let mut recovery_count = 0usize;
        let observer = |event: &Event<'_, _, _>| {
            let is_err = event.result().is_err();
            if is_err {
                recovery_count += 1;
                // Failed points are above threshold, so residual would be positive
                Some(Action::assume_positive())
            } else {
                None
            }
        };

        // Bracket: left residual at x=0 is 0-9=-9, right residual at x=3.5 is 12.25-9=3.25
        // Different signs, so valid bracket
        let solution = solve(&model, &problem, [0.0, 3.5], &Config::default(), observer)
            .expect("should recover and solve");

        assert_eq!(solution.status, Status::Converged);
        assert_relative_eq!(solution.x, 3.0, epsilon = 1e-10);
    }

    #[test]
    fn assume_residual_sign_discards_eval() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 9.0 };

        let observer = |event: &Event<'_, _, _>| match event {
            Event::Left { .. } => Some(Action::assume_negative()),
            Event::Right { .. } | Event::Midpoint { .. } => None,
        };

        let config = Config {
            max_iters: 0,
            ..Config::default()
        };

        let solution = solve(&model, &problem, [2.0, 10.0], &config, observer)
            .expect("should return best endpoint");

        assert_eq!(solution.status, Status::MaxIters);
        assert_relative_eq!(solution.x, 10.0);
    }

    #[test]
    fn errors_when_no_successful_evaluations() {
        let model = ThresholdModel { threshold: -1.0 };
        let problem = TargetOutputProblem { target: 9.0 };

        let observer = |event: &Event<'_, _, _>| match event {
            Event::Left { .. } => Some(Action::assume_negative()),
            Event::Right { .. } | Event::Midpoint { .. } => Some(Action::assume_positive()),
        };

        let result = solve(&model, &problem, [0.0, 10.0], &Config::default(), observer);

        assert!(matches!(result, Err(Error::NoSuccessfulEvaluation)));
    }

    #[test]
    fn converges_on_small_bracket_width() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 9.0 };

        let config = Config {
            max_iters: 10,
            x_abs_tol: 1.0,
            ..Config::default()
        };

        let solution = solve_unobserved(&model, &problem, [2.9, 3.1], &config)
            .expect("should converge on x tolerance");

        assert_eq!(solution.status, Status::Converged);
        assert_eq!(solution.iters, 0);
    }
}

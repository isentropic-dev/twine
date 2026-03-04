mod action;
mod config;
mod decision;
mod error;
mod eval_context;
mod event;
mod point;

pub use action::Action;
pub use config::{Config, ConfigError};
pub use error::Error;
pub use event::Event;
pub use point::Point;

pub use crate::equation::{
    bracket::{Bracket, BracketError, Sign},
    solution::{Solution, Status},
};

use twine_core::{EquationProblem, Model, Observer};

use crate::equation::{best::Best, bracket::Bounds, evaluate};

use decision::Decision;
use eval_context::EvalContext;

/// Finds a root of the equation using the bisection method.
///
/// Evaluates both endpoints to establish a bracket, then iterates by
/// evaluating midpoints and shrinking the bracket.
///
/// Endpoint evaluation failures are hard errors — if either endpoint fails,
/// the solver returns immediately with [`Error::Model`] or [`Error::Problem`].
/// For control over endpoint evaluation (e.g., domain-specific error
/// recovery or noise filtering), evaluate endpoints yourself and use
/// [`solve_from_bracket`].
///
/// # Observer
///
/// The observer receives an [`Event`] for each **midpoint** evaluation.
/// Endpoint evaluations are not observed.
/// See [`solve_from_bracket`] for details on observer actions.
///
/// # Errors
///
/// Returns an error if the bracket is invalid, the config is invalid,
/// or an endpoint or midpoint evaluation fails without observer recovery.
pub fn solve<M, P, Obs>(
    model: &M,
    problem: &P,
    bracket: [f64; 2],
    config: &Config,
    observer: Obs,
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

    // Evaluate left endpoint.
    let left_sign = match evaluate(model, problem, [left]) {
        Ok(eval) => {
            let sign = Sign::of(eval.residuals[0]);
            best.update(eval);
            sign
        }
        Err(error) => return Err(error.into()),
    };

    // Evaluate right endpoint.
    let right_sign = match evaluate(model, problem, [right]) {
        Ok(eval) => {
            let sign = Sign::of(eval.residuals[0]);
            best.update(eval);
            sign
        }
        Err(error) => return Err(error.into()),
    };

    // Validate bracket signs.
    let bracket = Bracket::from_bounds(bounds, left_sign, right_sign)?;

    solve_from_bracket_inner(model, problem, bracket, config, best, observer)
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

/// Finds a root using bisection with a pre-validated bracket.
///
/// This skips endpoint evaluation — the caller is responsible for evaluating
/// the endpoints and constructing a valid [`Bracket`] with known residual
/// signs.
/// This is useful when endpoint evaluation requires domain-specific handling
/// (e.g., error recovery, noise filtering) that the solver's observer protocol
/// doesn't cover.
///
/// # Observer
///
/// The observer receives an [`Event`] for each midpoint evaluation and may:
/// - Return [`Action::StopEarly`] to stop and return the best evaluation so far.
/// - Return [`Action::AssumeResidualSign`] to recover from evaluation failures
///   by providing a residual sign for bracket updates.
///   When this action is used on a successful evaluation, that evaluation is
///   not considered for the best solution.
///
/// # Notes
///
/// The returned [`Solution`] reflects the best successful midpoint evaluation
/// seen during the solve.
/// Endpoint evaluations are not tracked — if no midpoint succeeds, this
/// returns [`Error::NoSuccessfulEvaluation`].
///
/// # Errors
///
/// Returns an error if the config is invalid or the model or problem returns
/// an unrecovered error during evaluation.
pub fn solve_from_bracket<M, P, Obs>(
    model: &M,
    problem: &P,
    bracket: Bracket,
    config: &Config,
    observer: Obs,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    M::Input: Clone,
    M::Output: Clone,
    P: EquationProblem<1, Input = M::Input, Output = M::Output>,
    Obs: for<'a> Observer<Event<'a, M, P>, Action>,
{
    config.validate()?;
    solve_from_bracket_inner(model, problem, bracket, config, Best::empty(), observer)
}

/// Core midpoint loop shared by `solve` and `solve_from_bracket`.
fn solve_from_bracket_inner<M, P, Obs>(
    model: &M,
    problem: &P,
    mut bracket: Bracket,
    config: &Config,
    mut best: Best<M::Input, M::Output>,
    mut observer: Obs,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    M::Input: Clone,
    M::Output: Clone,
    P: EquationProblem<1, Input = M::Input, Output = M::Output>,
    Obs: for<'a> Observer<Event<'a, M, P>, Action>,
{
    if best.is_residual_converged(config.residual_tol) {
        return finish(best, Status::Converged, 0);
    }

    let mut ctx = EvalContext::new(model, problem, &mut observer);

    for iter in 1..=config.max_iters {
        if bracket.is_x_converged(config.x_abs_tol, config.x_rel_tol) {
            return finish(best, Status::Converged, iter - 1);
        }

        let mid = bracket.midpoint();
        let (mid_eval, mid_decision) = ctx.midpoint(mid, &bracket);
        if let Some(eval) = mid_eval {
            best.update(eval);
        }
        match mid_decision {
            Decision::Continue(sign) => bracket.shrink(mid, sign),
            Decision::StopEarly => {
                return finish(best, Status::StoppedByObserver, iter);
            }
            Decision::Error(error) => return Err(error),
        }

        if best.is_residual_converged(config.residual_tol) {
            return finish(best, Status::Converged, iter);
        }
    }

    finish(best, Status::MaxIters, config.max_iters)
}

/// Converts a best tracker into a solution or a "no successful evaluation" error.
fn finish<I, O>(best: Best<I, O>, status: Status, iters: usize) -> Result<Solution<I, O>, Error> {
    best.into_solution(status, iters)
        .ok_or(Error::NoSuccessfulEvaluation)
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
        type Error = Infallible;

        fn input(&self, x: &[f64; 1]) -> Result<Self::Input, Self::Error> {
            Ok(x[0])
        }

        fn residuals(
            &self,
            _input: &Self::Input,
            output: &Self::Output,
        ) -> Result<[f64; 1], Self::Error> {
            Ok([output - self.target])
        }
    }

    // --- solve tests (full lifecycle) ---

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
    fn solve_errors_on_endpoint_failure() {
        // Model fails everywhere — endpoints can't be evaluated.
        let model = ThresholdModel { threshold: -1.0 };
        let problem = TargetOutputProblem { target: 9.0 };

        let result = solve_unobserved(&model, &problem, [0.0, 10.0], &Config::default());

        assert!(matches!(result, Err(Error::Model(_))));
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
        assert_relative_eq!(solution.x, 2.0);
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

    // --- solve_from_bracket tests (midpoint loop only) ---

    #[test]
    fn from_bracket_finds_root() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 9.0 };

        // x=0: residual = 0-9 = -9 (negative)
        // x=10: residual = 100-9 = 91 (positive)
        let bracket =
            Bracket::new((0.0, Sign::Negative), (10.0, Sign::Positive)).expect("valid bracket");

        let solution = solve_from_bracket(&model, &problem, bracket, &Config::default(), ())
            .expect("should solve");

        assert_eq!(solution.status, Status::Converged);
        assert_relative_eq!(solution.x, 3.0, epsilon = 1e-10);
    }

    #[test]
    fn observer_can_stop_iteration() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 9.0 };

        let bracket =
            Bracket::new((0.0, Sign::Negative), (10.0, Sign::Positive)).expect("valid bracket");

        let mut eval_count = 0usize;
        let observer = |_event: &Event<'_, _, _>| {
            eval_count += 1;
            if eval_count >= 3 {
                Some(Action::StopEarly)
            } else {
                None
            }
        };

        let solution = solve_from_bracket(&model, &problem, bracket, &Config::default(), observer)
            .expect("should stop cleanly");

        assert_eq!(solution.status, Status::StoppedByObserver);
        assert_eq!(solution.iters, 3);
        assert_eq!(eval_count, 3);
    }

    #[test]
    fn midpoint_failure_assumes_sign() {
        // Model fails above x=3.5, root is at x=3 (for target=9)
        let model = ThresholdModel { threshold: 3.5 };
        let problem = TargetOutputProblem { target: 9.0 };

        // x=0: residual = -9 (negative), x=3.5: residual = 3.25 (positive)
        let bracket =
            Bracket::new((0.0, Sign::Negative), (3.5, Sign::Positive)).expect("valid bracket");

        let mut recovery_count = 0usize;
        let observer = |event: &Event<'_, _, _>| {
            if matches!(event, Event::ModelFailed { .. }) {
                recovery_count += 1;
                Some(Action::assume_positive())
            } else {
                None
            }
        };

        let solution = solve_from_bracket(&model, &problem, bracket, &Config::default(), observer)
            .expect("should recover and solve");

        assert_eq!(solution.status, Status::Converged);
        assert_relative_eq!(solution.x, 3.0, epsilon = 1e-10);
    }

    #[test]
    fn assume_residual_sign_discards_eval() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 9.0 };

        // x=2: residual = -5 (negative), x=10: residual = 91 (positive)
        let bracket =
            Bracket::new((2.0, Sign::Negative), (10.0, Sign::Positive)).expect("valid bracket");

        // Assume positive on every midpoint — always shrink from the right.
        // The actual eval is discarded, so best stays empty.
        let observer = |_event: &Event<'_, _, _>| Some(Action::assume_positive());

        let config = Config {
            max_iters: 3,
            ..Config::default()
        };

        let result = solve_from_bracket(&model, &problem, bracket, &config, observer);

        assert!(matches!(result, Err(Error::NoSuccessfulEvaluation)));
    }

    #[test]
    fn errors_when_no_successful_evaluations() {
        let model = ThresholdModel { threshold: -1.0 };
        let problem = TargetOutputProblem { target: 9.0 };

        let bracket =
            Bracket::new((0.0, Sign::Negative), (10.0, Sign::Positive)).expect("valid bracket");

        // Model fails everywhere, observer assumes signs to keep going.
        let observer = |_event: &Event<'_, _, _>| Some(Action::assume_positive());

        let result = solve_from_bracket(&model, &problem, bracket, &Config::default(), observer);

        assert!(matches!(result, Err(Error::NoSuccessfulEvaluation)));
    }
}

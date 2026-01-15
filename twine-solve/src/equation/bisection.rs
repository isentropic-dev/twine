mod config;
mod error;
mod solution;

pub use config::Config;
pub use error::Error;
pub use solution::{Solution, Status};

use crate::{
    equation::{EquationProblem, Evaluation, Observer, evaluate},
    model::Model,
};

/// Control actions supported by the bisection solver.
pub enum Action {
    /// Stop the solver early.
    ///
    /// This action is illustrative and may change once solver control
    /// patterns settle.
    StopEarly,
}

/// Iteration event emitted by the bisection solver.
pub struct Event<'a, I, O> {
    /// Iteration counter (1-based within the bisection loop).
    pub iter: usize,
    /// Current search bracket.
    pub bracket: [f64; 2],
    /// Evaluation at the current midpoint.
    pub eval: &'a Evaluation<I, O, 1>,
}

/// Finds a root of the equation using the bisection method.
/// Observers see each iteration's evaluation and bracket state.
///
/// # Errors
///
/// Returns an error if the bracket is invalid, the config is invalid,
/// or the model or problem returns an error during evaluation.
pub fn solve<I, O, Obs>(
    model: &impl Model<Input = I, Output = O>,
    problem: &impl EquationProblem<1, Input = I, Output = O>,
    bracket: [f64; 2],
    config: &Config,
    mut observer: Obs,
) -> Result<Solution<I, O>, Error>
where
    Obs: for<'a> Observer<Event<'a, I, O>, Action>,
{
    config
        .validate()
        .map_err(|reason| Error::InvalidConfig { reason })?;

    let (mut left, mut right) = validate_bracket(bracket)?;

    let left_eval = evaluate(model, problem, [left])?;
    let mut left_residual = left_eval.residuals[0];
    if !left_residual.is_finite() {
        return Err(Error::NonFiniteResidual {
            x: left,
            residual: left_residual,
        });
    }
    if left_residual.abs() <= config.residual_tol {
        return Ok(Solution::from_eval(left_eval, Status::Converged, 0));
    }

    let right_eval = evaluate(model, problem, [right])?;
    let right_residual = right_eval.residuals[0];
    if !right_residual.is_finite() {
        return Err(Error::NonFiniteResidual {
            x: right,
            residual: right_residual,
        });
    }
    if right_residual.abs() <= config.residual_tol {
        return Ok(Solution::from_eval(right_eval, Status::Converged, 0));
    }

    if left_residual.signum() == right_residual.signum() {
        return Err(Error::NoBracket {
            left,
            right,
            left_residual,
            right_residual,
        });
    }

    let (mut best, mut best_residual) = if left_residual.abs() <= right_residual.abs() {
        (left_eval, left_residual)
    } else {
        (right_eval, right_residual)
    };

    for iter in 1..=config.max_iters {
        let mid = 0.5 * (left + right);
        let mid_eval = evaluate(model, problem, [mid])?;
        let mid_residual = mid_eval.residuals[0];

        if !mid_residual.is_finite() {
            return Err(Error::NonFiniteResidual {
                x: mid,
                residual: mid_residual,
            });
        }

        let x_converged = (right - left).abs() <= config.x_abs_tol + config.x_rel_tol * mid.abs();
        let residual_converged = mid_residual.abs() <= config.residual_tol;
        let is_better = mid_residual.abs() < best_residual.abs();

        let event = Event {
            iter,
            bracket: [left, right],
            eval: &mid_eval,
        };

        if let Some(action) = observer.observe(&event) {
            match action {
                Action::StopEarly => {
                    let best_eval = if is_better { mid_eval } else { best };
                    return Ok(Solution::from_eval(
                        best_eval,
                        Status::StoppedByObserver,
                        iter,
                    ));
                }
            }
        }

        if x_converged || residual_converged {
            return Ok(Solution::from_eval(mid_eval, Status::Converged, iter));
        }

        if is_better {
            best = mid_eval;
            best_residual = mid_residual;
        }

        if left_residual.signum() == mid_residual.signum() {
            left = mid;
            left_residual = mid_residual;
        } else {
            right = mid;
        }
    }

    Ok(Solution::from_eval(
        best,
        Status::MaxIters,
        config.max_iters,
    ))
}

/// Runs bisection without observation.
///
/// # Errors
///
/// Returns an error if the bracket is invalid, the config is invalid,
/// or the model or problem returns an error during evaluation.
pub fn solve_unobserved<I, O>(
    model: &impl Model<Input = I, Output = O>,
    problem: &impl EquationProblem<1, Input = I, Output = O>,
    bracket: [f64; 2],
    config: &Config,
) -> Result<Solution<I, O>, Error> {
    solve(model, problem, bracket, config, ())
}

/// Validates bracket values and returns them in normalized (left < right) order.
fn validate_bracket(bracket: [f64; 2]) -> Result<(f64, f64), Error> {
    let [left, right] = bracket;

    if !left.is_finite() {
        return Err(Error::NonFiniteBracket { value: left });
    }

    if !right.is_finite() {
        return Err(Error::NonFiniteBracket { value: right });
    }

    #[allow(clippy::float_cmp)]
    if left == right {
        return Err(Error::ZeroWidthBracket { value: left });
    }

    if left < right {
        Ok((left, right))
    } else {
        Ok((right, left))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::Infallible;

    use approx::assert_relative_eq;

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

        let mut calls = 0usize;
        let observer = |event: &Event<'_, f64, f64>| {
            calls += 1;
            if event.iter >= 3 {
                Some(Action::StopEarly)
            } else {
                None
            }
        };

        let solution = solve(&model, &problem, [0.0, 10.0], &Config::default(), observer)
            .expect("should stop cleanly");

        assert_eq!(solution.status, Status::StoppedByObserver);
        assert_eq!(solution.iters, 3);
        assert_eq!(calls, 3);
    }

    #[test]
    fn normalizes_reversed_bracket() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 36.0 };

        // Bracket is reversed: [10.0, 0.0] instead of [0.0, 10.0]
        let solution = solve_unobserved(&model, &problem, [10.0, 0.0], &Config::default())
            .expect("should solve with reversed bracket");

        assert_eq!(solution.status, Status::Converged);
        assert_relative_eq!(solution.x, 6.0, epsilon = 1e-10);
    }

    #[test]
    fn errors_on_zero_width_bracket() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 25.0 };

        let result = solve_unobserved(&model, &problem, [5.0, 5.0], &Config::default());

        assert!(matches!(result, Err(Error::ZeroWidthBracket { .. })));
    }

    #[test]
    fn errors_on_non_finite_bracket() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 67.0 };

        let result = solve_unobserved(&model, &problem, [f64::NAN, 10.0], &Config::default());
        assert!(matches!(result, Err(Error::NonFiniteBracket { .. })));

        let result = solve_unobserved(&model, &problem, [0.0, f64::INFINITY], &Config::default());
        assert!(matches!(result, Err(Error::NonFiniteBracket { .. })));
    }

    #[test]
    fn errors_on_no_bracket() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 9.0 };

        // Both endpoints are positive (no sign change)
        let result = solve_unobserved(&model, &problem, [5.0, 10.0], &Config::default());

        assert!(matches!(result, Err(Error::NoBracket { .. })));
    }

    #[test]
    fn errors_on_invalid_config() {
        let model = SquareModel;
        let problem = TargetOutputProblem { target: 4.0 };

        let config = Config {
            x_abs_tol: -1.0,
            ..Config::default()
        };
        let result = solve_unobserved(&model, &problem, [0.0, 10.0], &config);
        assert!(matches!(result, Err(Error::InvalidConfig { .. })));
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
}

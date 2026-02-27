//! Capability traits for cross-solver observers.
//!
//! These traits abstract over solver-specific event and action types, enabling
//! observers to work generically across different solvers.
//!
//! # Event traits
//!
//! - [`HasResidual`] — events that carry a residual value
//! - [`HasObjective`] — events that carry an objective value
//!
//! # Action traits
//!
//! - [`CanStopEarly`] — actions that can signal early termination
//! - [`CanAssumeWorse`] — actions that can signal a worse-than-evaluated outcome
//!
//! # Example
//!
//! ```rust
//! use twine_core::Observer;
//! use twine_observers::traits::{CanStopEarly, HasResidual};
//!
//! struct GoodEnough {
//!     tolerance: f64,
//!     min_iters: usize,
//!     iter: usize,
//! }
//!
//! impl<E: HasResidual, A: CanStopEarly> Observer<E, A> for GoodEnough {
//!     fn observe(&mut self, event: &E) -> Option<A> {
//!         self.iter += 1;
//!         if self.iter >= self.min_iters && event.residual().abs() < self.tolerance {
//!             return Some(A::stop_early());
//!         }
//!         None
//!     }
//! }
//! ```

use twine_core::{EquationProblem, Model, OptimizationProblem};

use twine_solvers::{equation::bisection, optimization::golden_section, transient::euler};

/// An event that carries a residual value.
pub trait HasResidual {
    /// Returns the residual for this event.
    ///
    /// Returns `f64::NAN` when the event represents an error and no residual
    /// is available.
    fn residual(&self) -> f64;
}

/// An event that carries an objective value.
pub trait HasObjective {
    /// Returns the objective for this event.
    ///
    /// Returns `f64::NAN` when the event represents an error and no objective
    /// is available.
    fn objective(&self) -> f64;
}

/// An action type that can signal early termination.
pub trait CanStopEarly {
    /// Returns the action that stops the solver early.
    fn stop_early() -> Self;
}

/// An action type that can signal a worse-than-evaluated outcome.
pub trait CanAssumeWorse {
    /// Returns the action that treats this evaluation as worse than the other.
    fn assume_worse() -> Self;
}

// --- HasResidual for bisection::Event ---

impl<M, P> HasResidual for bisection::Event<'_, M, P>
where
    M: Model,
    P: EquationProblem<1, Input = M::Input, Output = M::Output>,
{
    fn residual(&self) -> f64 {
        match self.result() {
            Ok(eval) => eval.residuals[0],
            Err(_) => f64::NAN,
        }
    }
}

// --- HasObjective for golden_section::Event ---

impl<M, P> HasObjective for golden_section::Event<'_, M, P>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
{
    fn objective(&self) -> f64 {
        match self {
            golden_section::Event::Evaluated { point, .. } => point.objective,
            golden_section::Event::ModelFailed { .. }
            | golden_section::Event::ProblemFailed { .. } => f64::NAN,
        }
    }
}

// --- CanStopEarly impls ---

impl CanStopEarly for bisection::Action {
    fn stop_early() -> Self {
        Self::StopEarly
    }
}

impl CanStopEarly for golden_section::Action {
    fn stop_early() -> Self {
        Self::StopEarly
    }
}

impl CanStopEarly for euler::Action {
    fn stop_early() -> Self {
        Self::StopEarly
    }
}

// --- CanAssumeWorse for golden_section::Action ---

impl CanAssumeWorse for golden_section::Action {
    fn assume_worse() -> Self {
        Self::AssumeWorse
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use twine_core::{EquationProblem, Model, OptimizationProblem};
    use twine_solvers::optimization::golden_section::{self, Point};

    use super::{HasObjective, HasResidual};

    // --- Minimal stubs ---

    struct Identity;

    impl Model for Identity {
        type Input = f64;
        type Output = f64;
        type Error = Infallible;

        fn call(&self, input: &f64) -> Result<f64, Infallible> {
            Ok(*input)
        }
    }

    #[derive(Debug)]
    struct Failure;

    impl std::fmt::Display for Failure {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "failure")
        }
    }

    impl std::error::Error for Failure {}

    struct FailingModel;

    impl Model for FailingModel {
        type Input = f64;
        type Output = f64;
        type Error = Failure;

        fn call(&self, _: &f64) -> Result<f64, Failure> {
            Err(Failure)
        }
    }

    struct LinearProblem;

    impl EquationProblem<1> for LinearProblem {
        type Input = f64;
        type Output = f64;
        type Error = Infallible;

        fn input(&self, x: &[f64; 1]) -> Result<f64, Infallible> {
            Ok(x[0])
        }

        fn residuals(&self, _: &f64, output: &f64) -> Result<[f64; 1], Infallible> {
            Ok([*output])
        }
    }

    impl OptimizationProblem<1> for LinearProblem {
        type Input = f64;
        type Output = f64;
        type Error = Infallible;

        fn input(&self, x: &[f64; 1]) -> Result<f64, Infallible> {
            Ok(x[0])
        }

        fn objective(&self, _: &f64, output: &f64) -> Result<f64, Infallible> {
            Ok(*output)
        }
    }

    struct FailingOptProblem;

    impl OptimizationProblem<1> for FailingOptProblem {
        type Input = f64;
        type Output = f64;
        type Error = Failure;

        fn input(&self, x: &[f64; 1]) -> Result<f64, Failure> {
            Ok(x[0])
        }

        fn objective(&self, _: &f64, _: &f64) -> Result<f64, Failure> {
            Err(Failure)
        }
    }

    // --- HasResidual for bisection::Event ---

    #[test]
    fn bisection_residual_ok() {
        use twine_solvers::equation::bisection;

        // Drive the solver one step to get a real event with a valid residual.
        // LinearProblem: residual = output = input = x, so residual ≠ NAN.
        let model = Identity;
        let problem = LinearProblem;
        let mut residual_seen = None;
        let _ = bisection::solve(
            &model,
            &problem,
            [-1.0, 1.0],
            &bisection::Config::default(),
            |event: &bisection::Event<'_, Identity, LinearProblem>| {
                if residual_seen.is_none() {
                    residual_seen = Some(event.residual());
                }
                None
            },
        );
        let r = residual_seen.expect("at least one event emitted");
        assert!(r.is_finite(), "expected finite residual, got {r}");
    }

    #[test]
    fn bisection_residual_nan_on_model_error() {
        use twine_solvers::equation::bisection;

        // FailingModel always errors, so every event result is Err → NAN.
        let model = FailingModel;
        let problem = LinearProblem;
        let mut got_nan = false;
        let _ = bisection::solve(
            &model,
            &problem,
            [-1.0, 1.0],
            &bisection::Config::default(),
            |event: &bisection::Event<'_, FailingModel, LinearProblem>| {
                got_nan = event.residual().is_nan();
                Some(bisection::Action::StopEarly)
            },
        );
        assert!(got_nan);
    }

    // --- HasObjective for golden_section::Event ---

    #[test]
    fn golden_section_objective_evaluated() {
        let input = 1.0_f64;
        let output = 1.0_f64;
        let event: golden_section::Event<'_, Identity, LinearProblem> =
            golden_section::Event::Evaluated {
                point: Point::new(1.0, 7.5),
                input: &input,
                output: &output,
                other: Point::new(0.5, 4.0),
            };
        assert_eq!(event.objective(), 7.5);
    }

    #[test]
    fn golden_section_objective_nan_on_model_failed() {
        let error = Failure;
        let event: golden_section::Event<'_, FailingModel, LinearProblem> =
            golden_section::Event::ModelFailed {
                x: 0.5,
                other: Point::new(0.5, 1.0),
                error: &error,
            };
        assert!(event.objective().is_nan());
    }

    #[test]
    fn golden_section_objective_nan_on_problem_failed() {
        let error = Failure;
        let event: golden_section::Event<'_, Identity, FailingOptProblem> =
            golden_section::Event::ProblemFailed {
                x: 0.5,
                other: Point::new(0.5, 1.0),
                error: &error,
            };
        assert!(event.objective().is_nan());
    }
}

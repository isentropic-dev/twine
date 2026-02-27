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

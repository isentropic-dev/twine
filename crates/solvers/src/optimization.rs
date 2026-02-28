//! Solvers for optimization problems — minimizing or maximizing an objective.
//!
//! An [`OptimizationProblem`] maps solver variables `x: [f64; N]` to model
//! inputs, calls the model, and extracts a scalar objective. Solvers in this
//! module search for the `x` that minimizes or maximizes that objective.
//!
//! # Solvers
//!
//! - [`golden_section`] — derivative-free search over a bracketed interval for
//!   unimodal functions
//!
//! [`OptimizationProblem`]: twine_core::OptimizationProblem

mod evaluate;

pub use evaluate::{EvalError, EvaluateResult, Evaluation, evaluate};

pub mod golden_section;

//! Solvers for equation problems — finding roots of systems of equations.
//!
//! An [`EquationProblem`] maps solver variables `x: [f64; N]` to model inputs,
//! calls the model, and computes residuals. Solvers in this module drive those
//! residuals toward zero.
//!
//! # Solvers
//!
//! - [`bisection`] — guaranteed convergence on a bracketed interval
//!
//! [`EquationProblem`]: twine_core::EquationProblem

mod evaluate;

pub use evaluate::{EvalError, EvaluateResult, Evaluation, evaluate};

pub mod bisection;

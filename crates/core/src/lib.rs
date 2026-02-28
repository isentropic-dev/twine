//! Core traits and types for the Twine framework.
//!
//! This crate defines the shared abstractions that solvers, observers, and
//! models build on:
//!
//! - [`Model`] — a callable that maps a typed input to a typed output
//! - [`Snapshot`] — a captured input/output pair from a model call
//! - [`Observer`] — receives solver events and optionally returns control actions
//! - [`EquationProblem`], [`OptimizationProblem`], [`OdeProblem`] — problem
//!   traits that adapt solver variables to model inputs and extract metrics from
//!   outputs

mod model;
mod observer;
mod problems;
mod step;

pub use observer::Observer;
pub use problems::{EquationProblem, OdeProblem, OptimizationProblem};
pub use step::{DerivativeOf, StepIntegrable};
pub use {model::Model, model::Snapshot};

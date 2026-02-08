//! Golden section search for single-variable optimization.
//!
//! # Algorithm
//!
//! Golden section search finds the minimum (or maximum) of a unimodal function
//! on a bounded interval. It works by iteratively narrowing the search bracket
//! using the golden ratio (φ ≈ 1.618) to place interior evaluation points.
//!
//! # When to Use
//!
//! Golden section search is appropriate when:
//! - The objective function is unimodal (has a single local optimum) on the bracket
//! - Derivative information is unavailable or expensive
//! - Function evaluations are relatively cheap
//! - You need guaranteed convergence for continuous functions
//!
//! # Limitations
//!
//! - **Single variable only**: Works with [`OptimizationProblem<1>`]
//! - **Derivative-free**: Slower convergence than gradient-based methods
//! - **Unimodal assumption**: May find local optimum if multiple extrema exist

mod action;
mod bracket;
mod config;
mod error;
mod event;
mod search;
mod solution;

pub use action::Action;
pub use config::{Config, ConfigError};
pub use error::Error;
pub use event::Event;
pub use solution::Solution;

use twine_core::{Model, Observer, OptimizationProblem};

use search::search;

/// Finds the minimum of the objective using golden section search.
///
/// # Errors
///
/// Returns an error if the model or problem fails during evaluation.
pub fn minimize<M, P, Obs>(
    model: &M,
    problem: &P,
    bracket: [f64; 2],
    config: &Config,
    observer: Obs,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
    Obs: for<'a> Observer<Event<'a, M, P>, Action>,
{
    search(model, problem, bracket, config, observer, |v| v)
}

/// Finds the minimum of the objective without observer support.
///
/// This is a convenience wrapper around [`minimize`] that uses a no-op observer.
///
/// # Errors
///
/// Returns an error if the model or problem fails during evaluation.
pub fn minimize_unobserved<M, P>(
    model: &M,
    problem: &P,
    bracket: [f64; 2],
    config: &Config,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
{
    minimize(model, problem, bracket, config, ())
}

/// Finds the maximum of the objective using golden section search.
///
/// # Errors
///
/// Returns an error if the model or problem fails during evaluation.
pub fn maximize<M, P, Obs>(
    model: &M,
    problem: &P,
    bracket: [f64; 2],
    config: &Config,
    observer: Obs,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
    Obs: for<'a> Observer<Event<'a, M, P>, Action>,
{
    search(model, problem, bracket, config, observer, |v| -v)
}

/// Finds the maximum of the objective without observer support.
///
/// This is a convenience wrapper around [`maximize`] that uses a no-op observer.
///
/// # Errors
///
/// Returns an error if the model or problem fails during evaluation.
pub fn maximize_unobserved<M, P>(
    model: &M,
    problem: &P,
    bracket: [f64; 2],
    config: &Config,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
{
    maximize(model, problem, bracket, config, ())
}

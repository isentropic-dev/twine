//! Golden section search for single-variable optimization.
//!
//! # Algorithm
//!
//! Golden section search finds the minimum (or maximum) of a unimodal function
//! on a bounded interval. It maintains two interior points positioned by the
//! golden ratio, compares their objectives, and shrinks the bracket toward the
//! better point.
//!
//! # When to Use
//!
//! Golden section search is appropriate when:
//! - The objective function is unimodal (single optimum) on the bracket
//! - Derivative information is unavailable or expensive
//! - Function evaluations are relatively cheap
//! - You need guaranteed convergence for continuous functions
//!
//! # Limitations
//!
//! - **Single variable only**: Works with [`OptimizationProblem<1>`]
//! - **Derivative-free**: Slower convergence than gradient-based methods
//! - **Unimodal assumption**: May find local optimum if multiple extrema exist
//!
//! # Observer Events
//!
//! The solver emits one [`Event`] per evaluation after initialization:
//!
//! - [`Event::Evaluated`] — evaluation succeeded
//! - [`Event::ModelFailed`] — model returned an error
//! - [`Event::ProblemFailed`] — problem returned an error (input or objective)
//!
//! Each event includes `other`, the other interior point. In golden section
//! search, this is always the current best. During **initialization**, the
//! solver evaluates two points but emits only one event (for the second point),
//! since the first has no `other` yet.
//!
//! Observers can return [`Action::StopEarly`] to halt immediately, or
//! [`Action::AssumeWorse`] to treat the point as worse than `other` (useful for
//! error recovery or steering the search away from a region).

mod action;
mod bracket;
mod config;
mod error;
mod event;
mod init;
mod point;
mod search;
mod solution;
mod state;

#[cfg(test)]
mod tests;

pub use action::Action;
pub use config::{Config, ConfigError};
pub use error::Error;
pub use event::Event;
pub use point::Point;
pub use solution::{Solution, Status};

use twine_core::{Model, Observer, OptimizationProblem};

use search::search;

/// Finds the minimum of the objective using golden section search.
///
/// The observer receives an [`Event`] for each evaluation after the first.
/// See the [module docs](self) for details on event timing and observer actions.
///
/// # Errors
///
/// Returns an error if the model or problem fails during evaluation
/// and the observer does not return [`Action::AssumeWorse`] to recover.
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
/// The observer receives an [`Event`] for each evaluation after the first.
/// See the [module docs](self) for details on event timing and observer actions.
///
/// # Errors
///
/// Returns an error if the model or problem fails during evaluation
/// and the observer does not return [`Action::AssumeWorse`] to recover.
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

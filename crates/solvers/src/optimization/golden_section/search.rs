use twine_core::{Model, Observer, OptimizationProblem};

use super::{Action, Config, Error, Event, Solution};

/// Core golden section search implementation.
///
/// The `objective_transform` function is applied to objective values before
/// comparison, allowing the same algorithm to handle both minimization
/// (transform = identity) and maximization (transform = negation).
pub(super) fn search<M, P, Obs, F>(
    _model: &M,
    _problem: &P,
    _bracket: [f64; 2],
    _config: &Config,
    mut _observer: Obs,
    _objective_transform: F,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
    Obs: for<'a> Observer<Event<'a, M, P>, Action>,
    F: Fn(f64) -> f64,
{
    todo!("golden section search implementation")
}

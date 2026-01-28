use thiserror::Error;

use super::OptimizationProblem;
use twine_core::model::{Model, Snapshot};

/// The result of evaluating an optimization problem at a given `x`.
#[derive(Debug, Clone)]
pub struct Evaluation<I, O, const N: usize> {
    pub x: [f64; N],
    pub objective: f64,
    pub snapshot: Snapshot<I, O>,
}

/// Errors that can occur when evaluating an optimization problem.
#[derive(Debug, Error)]
pub enum EvalError<IE, ME, OE> {
    /// Failed to construct the model input from solver variables.
    #[error("failed to compute input")]
    Input(#[source] IE),
    /// The model call failed.
    #[error("model call failed")]
    Model(#[source] ME),
    /// Failed to compute the objective.
    #[error("failed to compute objective")]
    Objective(#[source] OE),
}

/// Type alias for the result of [`evaluate`].
pub type EvaluateResult<M, P, const N: usize> = Result<
    Evaluation<<M as Model>::Input, <M as Model>::Output, N>,
    EvalError<
        <P as OptimizationProblem<N>>::InputError,
        <M as Model>::Error,
        <P as OptimizationProblem<N>>::ObjectiveError,
    >,
>;

/// Evaluates the model in the context of an optimization problem.
///
/// This function maps `x` to model input, calls the model, then computes the
/// objective from the input and output.
///
/// # Errors
///
/// Returns an error if input mapping, model call, or objective computation fails.
pub fn evaluate<M, P, const N: usize>(
    model: &M,
    problem: &P,
    x: [f64; N],
) -> EvaluateResult<M, P, N>
where
    M: Model,
    P: OptimizationProblem<N, Input = M::Input, Output = M::Output>,
{
    let input = problem.input(&x).map_err(EvalError::Input)?;
    let output = model.call(&input).map_err(EvalError::Model)?;
    let objective = problem
        .objective(&input, &output)
        .map_err(EvalError::Objective)?;

    Ok(Evaluation {
        x,
        objective,
        snapshot: Snapshot::new(input, output),
    })
}

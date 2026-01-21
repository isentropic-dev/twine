use thiserror::Error;

use crate::equation::EquationProblem;
use twine_core::model::{Model, Snapshot};

/// The result of evaluating a model at a given `x`.
#[derive(Debug, Clone)]
pub struct Evaluation<I, O, const N: usize> {
    pub x: [f64; N],
    pub residuals: [f64; N],
    pub snapshot: Snapshot<I, O>,
}

/// Errors that can occur when evaluating a model at a given `x`.
#[derive(Debug, Error)]
pub enum EvalError<IE, ME, RE> {
    /// Failed to construct the model input from solver variables.
    #[error("failed to compute input")]
    Input(#[source] IE),
    /// The model call failed.
    #[error("model call failed")]
    Model(#[source] ME),
    /// Failed to compute residuals from the model output.
    #[error("failed to compute residual")]
    Residual(#[source] RE),
}

/// Type alias for the result of [`evaluate`].
pub type EvaluateResult<M, P, const N: usize> = Result<
    Evaluation<<M as Model>::Input, <M as Model>::Output, N>,
    EvalError<
        <P as EquationProblem<N>>::InputError,
        <M as Model>::Error,
        <P as EquationProblem<N>>::ResidualError,
    >,
>;

/// Evaluates the model at `x` and computes residuals.
///
/// # Errors
///
/// Returns an error if input mapping, model call, or residual computation fails.
pub fn evaluate<M, P, const N: usize>(
    model: &M,
    problem: &P,
    x: [f64; N],
) -> EvaluateResult<M, P, N>
where
    M: Model,
    P: EquationProblem<N, Input = M::Input, Output = M::Output>,
{
    let input = problem.input(&x).map_err(EvalError::Input)?;
    let output = model.call(&input).map_err(EvalError::Model)?;
    let residuals = problem
        .residuals(&input, &output)
        .map_err(EvalError::Residual)?;

    Ok(Evaluation {
        x,
        residuals,
        snapshot: Snapshot::new(input, output),
    })
}

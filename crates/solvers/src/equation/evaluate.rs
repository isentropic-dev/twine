use thiserror::Error;

use twine_core::{EquationProblem, Model, Snapshot};

/// The result of evaluating an equation problem at a given `x`.
#[derive(Debug, Clone)]
pub struct Evaluation<I, O, const N: usize> {
    pub x: [f64; N],
    pub residuals: [f64; N],
    pub snapshot: Snapshot<I, O>,
}

/// Errors that can occur when evaluating an equation problem.
#[derive(Debug, Error)]
pub enum EvalError<ME, PE> {
    /// The model call failed.
    #[error("model call failed")]
    Model(#[source] ME),
    /// Failed to construct input or compute residuals.
    #[error("problem error")]
    Problem(#[source] PE),
}

/// Type alias for the result of [`evaluate`].
pub type EvaluateResult<M, P, const N: usize> = Result<
    Evaluation<<M as Model>::Input, <M as Model>::Output, N>,
    EvalError<<M as Model>::Error, <P as EquationProblem<N>>::Error>,
>;

/// Evaluates the model in the context of an equation problem.
///
/// This function maps `x` to model input, calls the model, then computes
/// residuals from the input and output.
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
    let input = problem.input(&x).map_err(EvalError::Problem)?;
    let output = model.call(&input).map_err(EvalError::Model)?;
    let residuals = problem
        .residuals(&input, &output)
        .map_err(EvalError::Problem)?;

    Ok(Evaluation {
        x,
        residuals,
        snapshot: Snapshot::new(input, output),
    })
}

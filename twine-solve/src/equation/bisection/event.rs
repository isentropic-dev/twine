use crate::equation::{EquationProblem, EvaluateResult};
use twine_core::model::Model;

use super::Bracket;

/// Event emitted by the bisection solver for each evaluation.
pub enum Event<'a, M, P>
where
    M: Model,
    P: EquationProblem<1, Input = M::Input, Output = M::Output>,
{
    /// Left bracket endpoint evaluation.
    Left {
        /// The x value that was evaluated.
        x: f64,
        /// The result of the evaluation.
        result: &'a EvaluateResult<M, P, 1>,
    },
    /// Right bracket endpoint evaluation.
    Right {
        /// The x value that was evaluated.
        x: f64,
        /// The result of the evaluation.
        result: &'a EvaluateResult<M, P, 1>,
    },
    /// Midpoint evaluation with a validated bracket.
    Midpoint {
        /// The x value that was evaluated.
        x: f64,
        /// Current search bracket.
        bracket: &'a Bracket,
        /// The result of the evaluation.
        result: &'a EvaluateResult<M, P, 1>,
    },
}

impl<'a, M, P> Event<'a, M, P>
where
    M: Model,
    P: EquationProblem<1, Input = M::Input, Output = M::Output>,
{
    /// Returns the evaluated x value.
    #[must_use]
    pub fn x(&self) -> f64 {
        match self {
            Event::Left { x, .. } | Event::Right { x, .. } | Event::Midpoint { x, .. } => *x,
        }
    }

    /// Returns the evaluation result.
    pub fn result(&self) -> &'a EvaluateResult<M, P, 1> {
        match self {
            Event::Left { result, .. }
            | Event::Right { result, .. }
            | Event::Midpoint { result, .. } => result,
        }
    }
}

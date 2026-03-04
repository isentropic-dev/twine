use twine_core::{EquationProblem, Model};

use crate::equation::bracket::Bracket;

use super::Point;

/// Events emitted by the bisection solver during the midpoint loop.
///
/// Each event provides the evaluation outcome and a reference to the current
/// bracket.
/// Observers can pattern-match on the outcome (success vs. failure) to steer
/// the search.
pub enum Event<'a, M, P>
where
    M: Model,
    P: EquationProblem<1, Input = M::Input, Output = M::Output>,
{
    /// Successful evaluation.
    Evaluated {
        /// The evaluated point (x and residual).
        point: Point,

        /// The model input at this point.
        input: &'a M::Input,

        /// The model output at this point.
        output: &'a M::Output,

        /// The current search bracket.
        bracket: &'a Bracket,
    },

    /// Model evaluation failed.
    ModelFailed {
        /// The x value where evaluation failed.
        x: f64,

        /// The model error.
        error: &'a M::Error,

        /// The current search bracket.
        bracket: &'a Bracket,
    },

    /// Problem method failed (input construction or residual computation).
    ProblemFailed {
        /// The x value where evaluation failed.
        x: f64,

        /// The problem error.
        error: &'a P::Error,

        /// The current search bracket.
        bracket: &'a Bracket,
    },
}

impl<M, P> Event<'_, M, P>
where
    M: Model,
    P: EquationProblem<1, Input = M::Input, Output = M::Output>,
{
    /// Returns the x value that was evaluated (or attempted).
    #[must_use]
    pub fn x(&self) -> f64 {
        match self {
            Self::Evaluated { point, .. } => point.x,
            Self::ModelFailed { x, .. } | Self::ProblemFailed { x, .. } => *x,
        }
    }

    /// Returns the current search bracket.
    #[must_use]
    pub fn bracket(&self) -> &Bracket {
        match self {
            Self::Evaluated { bracket, .. }
            | Self::ModelFailed { bracket, .. }
            | Self::ProblemFailed { bracket, .. } => bracket,
        }
    }
}

use twine_core::{Model, Observer, OptimizationProblem};

use crate::optimization::evaluate::EvalError;

use super::{Action, Point};

/// Events emitted by the golden section solver.
///
/// Each event provides the current evaluation (or failure) and the `other`
/// interior point. In golden section search, `other` is always the current
/// best: the point the solver would keep if it had to choose now. Observers
/// can compare against `other` to decide whether to stop early or steer the
/// search with [`Action::AssumeWorse`].
pub enum Event<'a, M, P>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
{
    /// Successful evaluation of an interior point.
    Evaluated {
        /// The evaluated point (x and objective).
        point: Point,

        /// The model input at this point.
        input: &'a M::Input,

        /// The model output at this point.
        output: &'a M::Output,

        /// The other interior point.
        other: Point,
    },

    /// Model evaluation failed.
    ModelFailed {
        /// The x value where evaluation failed.
        x: f64,

        /// The other interior point.
        other: Point,

        /// The model error.
        error: &'a M::Error,
    },

    /// Problem method failed (input construction or objective computation).
    ProblemFailed {
        /// The x value where evaluation failed.
        x: f64,

        /// The other interior point.
        other: Point,

        /// The problem error.
        error: &'a P::Error,
    },
}

impl<M, P> Event<'_, M, P>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
{
    /// Returns the x value that was evaluated (or attempted).
    #[must_use]
    pub fn x(&self) -> f64 {
        match self {
            Self::Evaluated { point, .. } => point.x,
            Self::ModelFailed { x, .. } | Self::ProblemFailed { x, .. } => *x,
        }
    }

    /// Returns the other interior point.
    #[must_use]
    pub fn other(&self) -> Point {
        match self {
            Self::Evaluated { other, .. }
            | Self::ModelFailed { other, .. }
            | Self::ProblemFailed { other, .. } => *other,
        }
    }

    /// Emits a failure event and returns the observer's action.
    pub(super) fn emit_failure<Obs>(
        x: f64,
        other: Point,
        error: &EvalError<M::Error, P::Error>,
        observer: &mut Obs,
    ) -> Option<Action>
    where
        Obs: for<'a> Observer<Event<'a, M, P>, Action>,
    {
        match error {
            EvalError::Model(e) => {
                let event = Event::ModelFailed { x, other, error: e };
                observer.observe(&event)
            }
            EvalError::Problem(e) => {
                let event = Event::ProblemFailed { x, other, error: e };
                observer.observe(&event)
            }
        }
    }
}

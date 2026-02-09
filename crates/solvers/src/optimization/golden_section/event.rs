use twine_core::{Model, OptimizationProblem};

/// Events emitted by the golden section solver.
///
/// Each event includes the evaluated (or attempted) `x` and enough context for
/// observers to log progress, stop early, or recover from certain failures.
pub enum Event<'a, M, P>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
{
    /// Successful evaluation of a new interior point.
    Evaluated {
        /// The evaluated point (x and objective).
        point: Point,

        /// The model input at this point.
        input: &'a M::Input,

        /// The model output at this point.
        output: &'a M::Output,

        /// The other interior point (with known objective).
        other: Point,

        /// The best point seen so far (before considering this evaluation).
        best: Point,
    },

    /// Model evaluation failed.
    ModelFailed {
        /// The x value where evaluation failed.
        x: f64,

        /// The model input (successfully constructed).
        input: &'a M::Input,

        /// The other interior point.
        other: Point,

        /// The best point seen so far.
        best: Point,

        /// The model error.
        error: &'a M::Error,
    },

    /// Problem method failed (input construction or objective computation).
    ProblemFailed {
        /// The x value where evaluation failed.
        x: f64,

        /// The model input, if input construction succeeded.
        input: Option<&'a M::Input>,

        /// The model output, if model call succeeded.
        output: Option<&'a M::Output>,

        /// The other interior point.
        other: Point,

        /// The best point seen so far.
        best: Point,

        /// The problem error.
        error: &'a P::Error,
    },
}

/// A point with its evaluated objective value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// The x value.
    pub x: f64,

    /// The objective value at x.
    pub objective: f64,
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
            Event::Evaluated { point, .. } => point.x,
            Event::ModelFailed { x, .. } | Event::ProblemFailed { x, .. } => *x,
        }
    }

    /// Returns the other interior point.
    #[must_use]
    pub fn other(&self) -> Point {
        match self {
            Event::Evaluated { other, .. }
            | Event::ModelFailed { other, .. }
            | Event::ProblemFailed { other, .. } => *other,
        }
    }

    /// Returns the best point seen so far.
    #[must_use]
    pub fn best(&self) -> Point {
        match self {
            Event::Evaluated { best, .. }
            | Event::ModelFailed { best, .. }
            | Event::ProblemFailed { best, .. } => *best,
        }
    }
}

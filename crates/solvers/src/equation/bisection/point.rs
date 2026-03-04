use crate::equation::Evaluation;

/// A point with its evaluated residual value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// The x value.
    pub x: f64,

    /// The residual at x.
    pub residual: f64,
}

impl Point {
    /// Creates a new point.
    #[must_use]
    pub fn new(x: f64, residual: f64) -> Self {
        Self { x, residual }
    }
}

impl<I, O> From<&Evaluation<I, O, 1>> for Point {
    fn from(eval: &Evaluation<I, O, 1>) -> Self {
        Self::new(eval.x[0], eval.residuals[0])
    }
}

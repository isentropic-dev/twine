use crate::optimization::evaluate::Evaluation;

/// A point with its evaluated objective value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// The x value.
    pub x: f64,

    /// The objective value at x.
    pub objective: f64,
}

impl Point {
    /// Creates a new point.
    #[must_use]
    pub fn new(x: f64, objective: f64) -> Self {
        Self { x, objective }
    }
}

impl<I, O> From<&Evaluation<I, O, 1>> for Point {
    fn from(eval: &Evaluation<I, O, 1>) -> Self {
        Self::new(eval.x[0], eval.objective)
    }
}

/// Defines the optimization direction.
///
/// This trait enables zero-cost abstraction over minimization and maximization.
/// The solver transforms objective values using [`Goal::transform`], allowing
/// it to always minimize internally while supporting both directions.
pub trait Goal {
    /// Transforms an objective value for internal minimization.
    ///
    /// - [`Minimize`]: returns the value unchanged
    /// - [`Maximize`]: negates the value
    fn transform(value: f64) -> f64;
}

/// Minimize the objective function.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Minimize;

impl Goal for Minimize {
    #[inline]
    fn transform(value: f64) -> f64 {
        value
    }
}

/// Maximize the objective function.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Maximize;

impl Goal for Maximize {
    #[inline]
    fn transform(value: f64) -> f64 {
        -value
    }
}

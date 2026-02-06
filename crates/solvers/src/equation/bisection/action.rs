use super::bracket::Sign;

/// Control actions supported by the bisection solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    /// Stop the solver early and return the best solution found so far.
    StopEarly,

    /// Assume a residual sign for bracket updates.
    ///
    /// This action is mainly used for error recovery.
    /// If used on a successful evaluation, that evaluation is not considered
    /// for the best solution.
    AssumeResidualSign(Sign),
}

impl Action {
    /// Assumes a positive residual sign for bracket updates.
    #[must_use]
    pub fn assume_positive() -> Self {
        Self::AssumeResidualSign(Sign::Positive)
    }

    /// Assumes a negative residual sign for bracket updates.
    #[must_use]
    pub fn assume_negative() -> Self {
        Self::AssumeResidualSign(Sign::Negative)
    }
}

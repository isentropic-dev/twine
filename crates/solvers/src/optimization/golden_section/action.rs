/// Actions an observer can take during golden section search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Stop the solver early and return the best solution found so far.
    StopEarly,

    /// Treat this point as having a worse objective than the other point.
    ///
    /// This causes the solver to shrink away from this point.
    /// The evaluation (if successful) is not considered for the best solution.
    ///
    /// Use this for:
    /// - Recovering from model or problem errors when domain knowledge suggests
    ///   the failed region is suboptimal but the search should continue.
    /// - Steering the search away from a region even when evaluation succeeded.
    AssumeWorse,
}

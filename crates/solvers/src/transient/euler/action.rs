/// Control actions supported by the Euler solver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Stop the solver early and return the solution so far.
    StopEarly,
}

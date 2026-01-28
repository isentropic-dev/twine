use twine_core::model::Snapshot;

use super::Goal;

/// Defines an optimization problem to be solved.
pub trait OptimizationProblem<const N: usize> {
    type Goal: Goal;
    type Input;
    type Output;
    type InputError: std::error::Error + Send + Sync + 'static;
    type ObjectiveError: std::error::Error + Send + Sync + 'static;

    /// Maps solver variables (`x`) into a model input.
    ///
    /// # Errors
    ///
    /// Returns an error if the input cannot be constructed from `x`.
    fn input(&self, x: &[f64; N]) -> Result<Self::Input, Self::InputError>;

    /// Computes the objective value from model input/output.
    ///
    /// # Errors
    ///
    /// Returns an error if the objective cannot be computed.
    fn objective(
        &self,
        input: &Self::Input,
        output: &Self::Output,
    ) -> Result<f64, Self::ObjectiveError>;

    /// Computes the objective value directly from a snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if the objective cannot be computed.
    fn objective_from_snapshot(
        &self,
        snap: &Snapshot<Self::Input, Self::Output>,
    ) -> Result<f64, Self::ObjectiveError> {
        self.objective(&snap.input, &snap.output)
    }
}

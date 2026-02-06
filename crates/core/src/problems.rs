use crate::Snapshot;

/// Defines an equation (root-finding) problem to be solved.
pub trait EquationProblem<const N: usize> {
    type Input;
    type Output;
    type InputError: std::error::Error + Send + Sync + 'static;
    type ResidualError: std::error::Error + Send + Sync + 'static;

    /// Maps solver variables (`x`) into a model input.
    ///
    /// # Errors
    ///
    /// Returns an error if the input cannot be constructed from `x`.
    fn input(&self, x: &[f64; N]) -> Result<Self::Input, Self::InputError>;

    /// Computes residuals from model input/output.
    ///
    /// # Errors
    ///
    /// Returns an error if residuals cannot be computed.
    fn residuals(
        &self,
        input: &Self::Input,
        output: &Self::Output,
    ) -> Result<[f64; N], Self::ResidualError>;

    /// Computes residuals directly from a snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if residuals cannot be computed.
    fn residuals_from_snapshot(
        &self,
        snap: &Snapshot<Self::Input, Self::Output>,
    ) -> Result<[f64; N], Self::ResidualError> {
        self.residuals(&snap.input, &snap.output)
    }
}

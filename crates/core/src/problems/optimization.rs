/// Defines a minimization problem to be solved.
///
/// A minimization problem maps solver variables to a model input,
/// then computes an objective value from the model input and output.
/// Solvers search for the input that minimizes the objective.
///
/// The const generic `N` is the number of solver variables.
/// For example, `N = 1` represents a scalar minimization problem.
pub trait MinimizationProblem<const N: usize> {
    type Input;
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Maps solver variables (`x`) into a model input.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the input cannot be constructed from `x`.
    fn input(&self, x: &[f64; N]) -> Result<Self::Input, Self::Error>;

    /// Computes an objective value from model input/output.
    ///
    /// Solvers search for the input that minimizes this objective.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the objective cannot be computed.
    fn objective(
        &self,
        input: &Self::Input,
        output: &Self::Output,
    ) -> Result<f64, Self::Error>;
}

/// Defines a maximization problem to be solved.
///
/// A maximization problem maps solver variables to a model input,
/// then computes an objective value from the model input and output.
/// Solvers search for the input that maximizes the objective.
///
/// The const generic `N` is the number of solver variables.
/// For example, `N = 1` represents a scalar maximization problem.
pub trait MaximizationProblem<const N: usize> {
    type Input;
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Maps solver variables (`x`) into a model input.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the input cannot be constructed from `x`.
    fn input(&self, x: &[f64; N]) -> Result<Self::Input, Self::Error>;

    /// Computes an objective value from model input/output.
    ///
    /// Solvers search for the input that maximizes this objective.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the objective cannot be computed.
    fn objective(
        &self,
        input: &Self::Input,
        output: &Self::Output,
    ) -> Result<f64, Self::Error>;
}

/// Defines an equation (root-finding) problem to be solved.
///
/// An equation problem maps solver variables to a model input,
/// then computes residuals from the model input and output.
/// Solvers drive the residuals toward zero.
///
/// The const generic `N` is the number of solver variables and residuals.
/// For systems of equations, `N` is the number of equations to solve
/// simultaneously. For example, `N = 1` represents a scalar problem.
pub trait EquationProblem<const N: usize> {
    type Input;
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Maps solver variables (`x`) into a model input.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the input cannot be constructed from `x`.
    fn input(&self, x: &[f64; N]) -> Result<Self::Input, Self::Error>;

    /// Computes residuals from model input/output.
    ///
    /// Solvers drive these residuals toward zero.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if residuals cannot be computed.
    fn residuals(
        &self,
        input: &Self::Input,
        output: &Self::Output,
    ) -> Result<[f64; N], Self::Error>;
}

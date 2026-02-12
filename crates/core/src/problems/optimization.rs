/// Defines an optimization problem to be solved.
///
/// An optimization problem maps solver variables to a model input,
/// then computes a scalar objective value from the model input and output.
///
/// The direction of optimization (minimize or maximize) is determined by
/// the solver function chosen (e.g., [`golden_section::minimize`][gs-min] vs
/// [`golden_section::maximize`][gs-max]).
///
/// The const generic `N` is the number of solver variables.
/// For example, `N = 1` represents a scalar optimization problem.
///
/// [gs-min]: https://docs.rs/twine-solvers/latest/twine_solvers/optimization/golden_section/fn.minimize.html
/// [gs-max]: https://docs.rs/twine-solvers/latest/twine_solvers/optimization/golden_section/fn.maximize.html
pub trait OptimizationProblem<const N: usize> {
    type Input;
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Maps solver variables (`x`) into a model input.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the input cannot be constructed from `x`.
    fn input(&self, x: &[f64; N]) -> Result<Self::Input, Self::Error>;

    /// Computes a scalar objective value from model input/output.
    ///
    /// The solver determines whether to minimize or maximize this objective.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the objective cannot be computed.
    fn objective(&self, input: &Self::Input, output: &Self::Output) -> Result<f64, Self::Error>;
}

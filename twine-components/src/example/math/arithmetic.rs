use twine_core::Component;

/// A component that performs basic arithmetic on two values.
///
/// Given two `f64` values, `x` and `y`, this component computes:
///
/// - **Sum**: `x + y`
/// - **Difference**: `x - y`
/// - **Product**: `x * y`
/// - **Quotient**: `x / y`
/// - **Average**: `(x + y) / 2`
///
/// # Panics
///
/// Panics if `y` is zero when computing the quotient.
///
/// In the future, this component will return a `Result` instead of panicking,
/// as part of the broader error handling design work being tracked at
/// <https://github.com/isentropic-dev/twine/issues/41>.
pub struct Arithmetic;

/// The input for the [`Arithmetic`] component.
pub struct ArithmeticInput {
    pub x: f64,
    pub y: f64,
}

/// The output for the [`Arithmetic`] component.
pub struct ArithmeticOutput {
    pub sum: f64,
    pub difference: f64,
    pub product: f64,
    pub quotient: f64,
    pub average: f64,
}

impl Component for Arithmetic {
    type Input = ArithmeticInput;
    type Output = ArithmeticOutput;

    fn call(&self, input: Self::Input) -> Self::Output {
        let Self::Input { x, y } = input;

        Self::Output {
            sum: x + y,
            difference: x - y,
            product: x * y,
            quotient: x / y,
            average: (x + y) * 0.5,
        }
    }
}

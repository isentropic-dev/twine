use std::fmt;

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
/// # Errors
///
/// Returns [`ArithmeticError::DivisionByZero`] if `y` is zero.
pub struct Arithmetic;

/// The input for the [`Arithmetic`] component.
pub struct ArithmeticInput {
    pub x: f64,
    pub y: f64,
}

/// The output for the [`Arithmetic`] component.
#[derive(Debug, PartialEq)]
pub struct ArithmeticOutput {
    pub sum: f64,
    pub difference: f64,
    pub product: f64,
    pub quotient: f64,
    pub average: f64,
}

/// An error type for the [`Arithmetic`] component.
#[derive(Debug, PartialEq)]
pub enum ArithmeticError {
    DivisionByZero,
}

impl Component for Arithmetic {
    type Input = ArithmeticInput;
    type Output = ArithmeticOutput;
    type Error = ArithmeticError;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let Self::Input { x, y } = input;

        if y == 0.0 {
            Err(ArithmeticError::DivisionByZero)
        } else {
            Ok(Self::Output {
                sum: x + y,
                difference: x - y,
                product: x * y,
                quotient: x / y,
                average: (x + y) * 0.5,
            })
        }
    }
}

impl fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticError::DivisionByZero => {
                write!(f, "Value for y must not be zero.")
            }
        }
    }
}

impl std::error::Error for ArithmeticError {}

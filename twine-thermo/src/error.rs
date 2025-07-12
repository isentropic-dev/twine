use thiserror::Error;

/// Errors that may occur when evaluating thermodynamic properties.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PropertyError {
    /// The property is not supported by this model.
    ///
    /// Indicates that the model does not implement the property at all,
    /// regardless of the state.
    #[error("property `{property}` is not implemented by this model")]
    NotImplemented {
        property: &'static str,
        context: Option<String>,
    },

    /// The property is undefined at the given state.
    ///
    /// For example, the specific heat capacity of a pure fluid within the vapor dome.
    #[error("property `{property}` is undefined at the given state")]
    Undefined {
        property: &'static str,
        context: Option<String>,
    },

    /// The input values are invalid or inconsistent.
    ///
    /// Indicates that the inputs are physically invalid or outside the model's valid domain.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// The calculation failed due to a numerical or internal error.
    ///
    /// For example, division by zero or a failure to converge.
    #[error("calculation error: {0}")]
    Calculation(String),
}

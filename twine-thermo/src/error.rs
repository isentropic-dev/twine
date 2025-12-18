use thiserror::Error;

/// Errors that may occur when evaluating thermodynamic properties.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PropertyError {
    /// The property is undefined at the given state.
    ///
    /// For example, the specific heat capacity of a pure fluid within the vapor dome.
    #[error("undefined property: {context}")]
    Undefined { context: String },

    /// The input state is outside the model's valid domain.
    #[error("out of domain: {context}")]
    OutOfDomain { context: String },

    /// The provided state is invalid or inconsistent.
    #[error("invalid state: {context}")]
    InvalidState { context: String },

    /// The calculation failed due to a numerical or internal error.
    ///
    /// For example, division by zero or a failure to converge.
    #[error("calculation error: {context}")]
    Calculation { context: String },
}
